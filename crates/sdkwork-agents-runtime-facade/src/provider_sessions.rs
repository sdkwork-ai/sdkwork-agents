use std::collections::{HashMap, HashSet};

use sdkwork_agent_kernel::{AgentMessage, AgentSession, SessionKind};
use sdkwork_agent_provider_core::{
    normalize_provider_session_path, provider_session_directory_fingerprint,
    provider_session_path_basename,
};

use crate::agent_engines::AgentEngineSlot;
use crate::error::{RuntimeFacadeError, RuntimeFacadeResult};

/// Prevent a corrupt provider history directory from retaining an unbounded
/// cross-provider inventory in the runtime process. Callers receive an
/// explicit failure and can narrow the project selector before retrying.
const MAX_PROVIDER_SESSION_INVENTORY_ITEMS: usize = 10_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderSessionProjectCwdSelector {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub owner_user_id: u64,
    pub project_id: String,
    pub project_name: String,
}

pub trait ProviderSessionProjectCwdResolver: Send + Sync {
    fn resolve_project_cwd(
        &self,
        selector: &ProviderSessionProjectCwdSelector,
    ) -> RuntimeFacadeResult<Option<String>>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderSessionInventorySelector {
    pub directory_fingerprint: Option<String>,
    pub exact_cwd: Option<String>,
    pub unique_basename: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderSessionInventoryItem {
    pub engine_key: String,
    pub agent_id: String,
    pub binding_id: String,
    pub provider_id: String,
    pub default_model_id: String,
    pub directory: ProviderSessionDirectoryEntry,
    pub session: AgentSession,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderSessionInventoryIssue {
    pub engine_key: String,
    pub reason: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProviderSessionInventorySnapshot {
    pub directory_resolved: bool,
    pub items: Vec<ProviderSessionInventoryItem>,
    pub successful_engine_keys: Vec<String>,
    pub issues: Vec<ProviderSessionInventoryIssue>,
    /// True when provider Sessions exist in the runtime inventory but none
    /// could be attributed to the requested project directory (the selector
    /// resolved no exact cwd and no unique basename match). Consumers must
    /// surface this instead of silently reporting an empty project.
    pub unattributed_provider_sessions: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderSessionDirectoryEntry {
    pub title: Option<String>,
    pub title_source: Option<String>,
    pub preview: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub recency_at: Option<String>,
    pub pinned: bool,
    pub archived: bool,
    pub visible: bool,
    pub source: Option<String>,
    pub sort_key: String,
}

impl Default for ProviderSessionDirectoryEntry {
    fn default() -> Self {
        Self {
            title: None,
            title_source: None,
            preview: None,
            created_at: None,
            updated_at: None,
            recency_at: None,
            pinned: false,
            archived: false,
            visible: true,
            source: None,
            sort_key: String::new(),
        }
    }
}

impl ProviderSessionDirectoryEntry {
    fn from_session(session: &AgentSession) -> Self {
        Self {
            title: metadata_or(
                session,
                "sdkwork.provider.session.directory.title",
                session.title.as_deref(),
            ),
            title_source: metadata_or(
                session,
                "sdkwork.provider.session.directory.title_source",
                Some("provider"),
            ),
            preview: metadata_or(
                session,
                "sdkwork.provider.session.directory.preview",
                session.preview.as_deref(),
            ),
            created_at: metadata_or(
                session,
                "sdkwork.provider.session.directory.created_at",
                session.created_at.as_deref(),
            ),
            updated_at: metadata_or(
                session,
                "sdkwork.provider.session.directory.updated_at",
                session.updated_at.as_deref(),
            ),
            recency_at: metadata_or(
                session,
                "sdkwork.provider.session.directory.recency_at",
                session.updated_at.as_deref(),
            ),
            pinned: metadata_bool(session, "sdkwork.provider.session.directory.pinned", false),
            archived: metadata_bool(
                session,
                "sdkwork.provider.session.directory.archived",
                false,
            ),
            visible: metadata_bool(session, "sdkwork.provider.session.directory.visible", true),
            source: metadata_or(session, "sdkwork.provider.session.directory.source", None),
            sort_key: metadata_or(session, "sdkwork.provider.session.directory.sort_key", None)
                .unwrap_or_else(|| descending_lexical_sort_key(&session.session_id)),
        }
    }
}

fn descending_lexical_sort_key(value: &str) -> String {
    let value = value.trim();
    let mut key = String::with_capacity(value.len().saturating_mul(2).saturating_add(2));
    for byte in value.as_bytes() {
        use std::fmt::Write;
        let _ = write!(key, "{:02x}", !byte);
    }
    key.push_str("ff");
    key
}

fn metadata_or(session: &AgentSession, key: &str, fallback: Option<&str>) -> Option<String> {
    session
        .metadata_value(key)
        .or(fallback)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn metadata_bool(session: &AgentSession, key: &str, fallback: bool) -> bool {
    session
        .metadata_value(key)
        .and_then(|value| value.parse::<bool>().ok())
        .unwrap_or(fallback)
}

pub(crate) fn discover_provider_sessions(
    slots: &HashMap<String, AgentEngineSlot>,
    selector: &ProviderSessionInventorySelector,
) -> RuntimeFacadeResult<ProviderSessionInventorySnapshot> {
    discover_provider_sessions_with(
        slots,
        selector,
        |_, slot| slot.list_provider_sessions_for_directory(selector.exact_cwd.as_deref()),
        |engine_key, slot| {
            let default_model = slot
                .list_model_descriptors()
                .into_iter()
                .next()
                .ok_or_else(|| {
                    format!("agent engine {engine_key} did not publish a model provider")
                })?;
            let agent_id =
                crate::agent_engines::agent_engine_agent_id(engine_key).ok_or_else(|| {
                    format!("agent engine {engine_key} did not publish an agent identity")
                })?;
            Ok((
                agent_id.to_string(),
                default_model.provider_id,
                default_model.model_id,
            ))
        },
    )
}

fn discover_provider_sessions_with(
    slots: &HashMap<String, AgentEngineSlot>,
    selector: &ProviderSessionInventorySelector,
    mut list_sessions: impl FnMut(
        &str,
        &AgentEngineSlot,
    ) -> sdkwork_agent_kernel::KernelResult<Vec<AgentSession>>,
    mut resolve_identity: impl FnMut(&str, &AgentEngineSlot) -> Result<(String, String, String), String>,
) -> RuntimeFacadeResult<ProviderSessionInventorySnapshot> {
    let mut candidates = Vec::new();
    let mut successful_engine_keys = Vec::new();
    let mut issues = Vec::new();
    for (engine_key, slot) in slots {
        let sessions = match list_sessions(engine_key, slot) {
            Ok(sessions) => sessions,
            Err(error) => {
                issues.push(ProviderSessionInventoryIssue {
                    engine_key: engine_key.clone(),
                    reason: error.to_string(),
                });
                continue;
            }
        };
        let (agent_id, provider_id, default_model_id) = match resolve_identity(engine_key, slot) {
            Ok(identity) => identity,
            Err(reason) => {
                issues.push(ProviderSessionInventoryIssue {
                    engine_key: engine_key.clone(),
                    reason,
                });
                continue;
            }
        };
        successful_engine_keys.push(engine_key.clone());
        for session in sessions {
            if candidates.len() >= MAX_PROVIDER_SESSION_INVENTORY_ITEMS {
                return Err(RuntimeFacadeError::InvalidInput(format!(
                    "provider session inventory exceeds {MAX_PROVIDER_SESSION_INVENTORY_ITEMS} items"
                )));
            }
            let directory = ProviderSessionDirectoryEntry::from_session(&session);
            candidates.push(ProviderSessionInventoryItem {
                engine_key: engine_key.to_string(),
                agent_id: agent_id.clone(),
                binding_id: slot.binding_id().to_string(),
                provider_id: provider_id.clone(),
                default_model_id: default_model_id.clone(),
                directory,
                session,
            });
        }
    }

    let selected_cwd = resolve_selected_cwd(&candidates, selector)?;
    successful_engine_keys.sort();
    issues.sort_by(|left, right| left.engine_key.cmp(&right.engine_key));
    let unattributed_provider_sessions = selected_cwd.is_none() && !candidates.is_empty();
    Ok(ProviderSessionInventorySnapshot {
        directory_resolved: selected_cwd.is_some(),
        items: select_top_level_provider_sessions(candidates, selected_cwd.as_deref()),
        successful_engine_keys,
        issues,
        unattributed_provider_sessions,
    })
}

fn select_top_level_provider_sessions(
    candidates: Vec<ProviderSessionInventoryItem>,
    selected_cwd: Option<&str>,
) -> Vec<ProviderSessionInventoryItem> {
    // Without a resolved project directory no provider session can be attributed
    // to the project. Never fall back to sessions whose cwd is unknown (provider
    // adapters such as openclaw/hermes do not populate cwd): selecting them here
    // would leak every runtime's sessions into every project whose directory
    // could not be resolved, making each project's Session list wrong.
    if selected_cwd.is_none() {
        return Vec::new();
    }
    let mut dedupe = HashSet::new();
    let mut selected = candidates
        .into_iter()
        .filter(|item| {
            item.session
                .cwd
                .as_deref()
                .map(normalize_provider_session_path)
                .as_deref()
                == selected_cwd
        })
        .filter(|item| {
            item.session.kind != SessionKind::Subagent && item.session.parent_session_id.is_none()
        })
        .filter(|item| item.directory.visible && !item.directory.archived)
        .filter(|item| {
            dedupe.insert((
                item.binding_id.trim().to_string(),
                item.provider_id.trim().to_string(),
                item.session.session_id.trim().to_string(),
            ))
        })
        .collect::<Vec<_>>();
    selected.sort_by(|left, right| {
        right
            .directory
            .pinned
            .cmp(&left.directory.pinned)
            .then_with(|| {
                right
                    .directory
                    .recency_at
                    .as_deref()
                    .unwrap_or_default()
                    .cmp(left.directory.recency_at.as_deref().unwrap_or_default())
            })
            .then_with(|| left.engine_key.cmp(&right.engine_key))
            .then_with(|| left.directory.sort_key.cmp(&right.directory.sort_key))
    });
    selected
}

pub(crate) fn load_provider_session_messages(
    slots: &HashMap<String, AgentEngineSlot>,
    engine_key: &str,
    provider_session_id: &str,
) -> RuntimeFacadeResult<Vec<AgentMessage>> {
    load_provider_session_messages_for_directory(slots, engine_key, provider_session_id, None)
}

pub(crate) fn load_provider_session_messages_for_directory(
    slots: &HashMap<String, AgentEngineSlot>,
    engine_key: &str,
    provider_session_id: &str,
    working_directory: Option<&str>,
) -> RuntimeFacadeResult<Vec<AgentMessage>> {
    let Some(slot) = slots.get(engine_key) else {
        return Err(RuntimeFacadeError::UnsupportedEngine {
            engine_key: engine_key.to_string(),
        });
    };
    if provider_session_id.trim().is_empty() {
        return Err(RuntimeFacadeError::InvalidInput(
            "provider session id is required to load transcript messages".to_string(),
        ));
    }
    slot.get_provider_session_history_for_directory(provider_session_id, working_directory)
        .map_err(|error| RuntimeFacadeError::EngineUnavailable {
            engine_key: engine_key.to_string(),
            reason: error.to_string(),
        })
}

pub(crate) fn load_provider_session_children(
    slots: &HashMap<String, AgentEngineSlot>,
    engine_key: &str,
    provider_session_id: &str,
    working_directory: Option<&str>,
) -> RuntimeFacadeResult<Vec<String>> {
    let Some(slot) = slots.get(engine_key) else {
        return Err(RuntimeFacadeError::UnsupportedEngine {
            engine_key: engine_key.to_string(),
        });
    };
    if provider_session_id.trim().is_empty() {
        return Err(RuntimeFacadeError::InvalidInput(
            "provider session id is required to enumerate sub-agent sessions".to_string(),
        ));
    }
    slot.list_provider_session_children(provider_session_id, working_directory)
        .map_err(|error| RuntimeFacadeError::EngineUnavailable {
            engine_key: engine_key.to_string(),
            reason: error.to_string(),
        })
}

fn resolve_selected_cwd(
    candidates: &[ProviderSessionInventoryItem],
    selector: &ProviderSessionInventorySelector,
) -> RuntimeFacadeResult<Option<String>> {
    if let Some(cwd) = selector
        .exact_cwd
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        return Ok(Some(normalize_provider_session_path(cwd)));
    }
    let Some(basename) = selector
        .unique_basename
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Err(RuntimeFacadeError::InvalidInput(
            "provider session inventory requires an exact cwd or unique basename".to_string(),
        ));
    };
    let basename = basename.to_ascii_lowercase();
    if let Some(directory_fingerprint) = selector
        .directory_fingerprint
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let matching_paths = candidates
            .iter()
            .filter_map(|item| item.session.cwd.as_deref())
            .filter(|cwd| provider_session_path_basename(cwd).as_deref() == Some(basename.as_str()))
            .filter(|cwd| {
                provider_session_directory_fingerprint(cwd).ok().as_deref()
                    == Some(directory_fingerprint)
            })
            .map(normalize_provider_session_path)
            .collect::<HashSet<_>>();
        return match matching_paths.len() {
            0 => Ok(None),
            1 => Ok(matching_paths.into_iter().next()),
            _ => Err(RuntimeFacadeError::InvalidInput(format!(
                "provider session directory fingerprint is ambiguous: {basename}"
            ))),
        };
    }
    let matching_paths = candidates
        .iter()
        .filter_map(|item| item.session.cwd.as_deref())
        .filter(|cwd| provider_session_path_basename(cwd).as_deref() == Some(basename.as_str()))
        .map(normalize_provider_session_path)
        .collect::<HashSet<_>>();
    match matching_paths.len() {
        0 => Ok(None),
        1 => Ok(matching_paths.into_iter().next()),
        _ => Err(RuntimeFacadeError::InvalidInput(format!(
            "provider session directory name is ambiguous: {basename}"
        ))),
    }
}

#[cfg(test)] // WORKSPACE-PATH:allow-fixture-block: this module is the file's #[cfg(test)] unit-test fixture data
mod tests {
    use super::*;
    use crate::agent_engines::bootstrap_agent_engine;

    fn item(engine: &str, session_id: &str, cwd: &str) -> ProviderSessionInventoryItem {
        let session = AgentSession::new(session_id).with_cwd(cwd);
        ProviderSessionInventoryItem {
            engine_key: engine.to_string(),
            agent_id: format!("agent.{engine}"),
            binding_id: format!("binding.{engine}"),
            provider_id: format!("provider.{engine}"),
            default_model_id: "model.default".to_string(),
            directory: ProviderSessionDirectoryEntry::from_session(&session),
            session,
        }
    }

    #[test]
    fn exact_windows_path_matches_extended_length_provider_cwd() {
        let candidates = vec![item("codex", "one", r"\\?\E:\Work\BirdCoder")];
        let selected = resolve_selected_cwd(
            &candidates,
            &ProviderSessionInventorySelector {
                directory_fingerprint: None,
                exact_cwd: Some("e:/work/birdcoder/".to_string()),
                unique_basename: None,
            },
        )
        .expect("selected cwd");
        assert_eq!(selected.as_deref(), Some("e:/work/birdcoder"));
    }

    #[test]
    fn basename_selection_fails_closed_when_multiple_paths_match() {
        let candidates = vec![
            item("codex", "one", "C:/one/BirdCoder"),
            item("opencode", "two", "D:/two/BirdCoder"),
        ];
        let result = resolve_selected_cwd(
            &candidates,
            &ProviderSessionInventorySelector {
                directory_fingerprint: None,
                exact_cwd: None,
                unique_basename: Some("BirdCoder".to_string()),
            },
        );
        assert!(matches!(result, Err(RuntimeFacadeError::InvalidInput(_))));
    }

    #[test]
    fn top_level_inventory_deduplicates_runtime_scoped_provider_identity_and_excludes_subagents() {
        let root = item("codex", "root-session", r"E:\Work\BirdCoder");
        let mut duplicate = item("codex", " root-session ", r"E:\Work\BirdCoder");
        duplicate.provider_id = root.provider_id.clone();
        let mut subagent = item("codex", "child-session", r"E:\Work\BirdCoder");
        subagent.session.kind = SessionKind::Subagent;
        subagent.session.parent_session_id = Some("root-session".to_string());

        let selected = select_top_level_provider_sessions(
            vec![root, duplicate, subagent],
            Some("e:/work/birdcoder"),
        );

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].session.session_id, "root-session");
    }

    #[test]
    fn top_level_inventory_keeps_the_same_provider_session_from_distinct_runtime_bindings() {
        let root = item("codex", "shared-session", r"E:\Work\BirdCoder");
        let mut distinct_runtime = item("opencode", "shared-session", r"E:\Work\BirdCoder");
        distinct_runtime.provider_id = root.provider_id.clone();

        let selected = select_top_level_provider_sessions(
            vec![root, distinct_runtime],
            Some("e:/work/birdcoder"),
        );

        assert_eq!(selected.len(), 2);
        assert_ne!(selected[0].binding_id, selected[1].binding_id);
    }

    #[test]
    fn unresolved_project_directory_never_selects_cwdless_provider_sessions() {
        // openclaw/hermes adapters do not populate AgentSession.cwd. When the
        // project directory cannot be resolved (selected_cwd is None), those
        // sessions must NOT be attributed to the project, otherwise every
        // project would receive the same cross-project Session list.
        let mut cwdless = item("openclaw", "global-session", "");
        cwdless.session.cwd = None;
        let root = item("codex", "root-session", r"E:\Work\BirdCoder");

        let selected = select_top_level_provider_sessions(vec![cwdless, root], None);

        assert_eq!(selected.len(), 0);
    }

    #[test]
    fn resolved_project_directory_keeps_cwdless_provider_sessions_out() {
        let mut cwdless = item("openclaw", "global-session", "");
        cwdless.session.cwd = None;
        let root = item("codex", "root-session", r"E:\Work\BirdCoder");

        let selected =
            select_top_level_provider_sessions(vec![cwdless, root], Some("e:/work/birdcoder"));

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].session.session_id, "root-session");
    }

    #[test]
    fn descending_lexical_sort_key_preserves_descending_provider_ids() {
        let mut values = ["thread-a", "thread-aa", "thread-b"];
        values.sort_by_key(|value| descending_lexical_sort_key(value));
        assert_eq!(values, ["thread-b", "thread-aa", "thread-a"]);
    }

    #[test]
    fn registered_gemini_engine_routes_provider_history_through_the_slot() {
        let mut slots = HashMap::new();
        slots.insert(
            "gemini".to_string(),
            bootstrap_agent_engine("gemini").expect("Gemini bootstrap"),
        );

        load_provider_session_messages(&slots, "gemini", "session-does-not-exist")
            .expect("Gemini history discovery should be supported");
    }

    #[test]
    fn one_provider_inventory_failure_does_not_hide_working_provider_sessions() {
        let mut slots = HashMap::new();
        slots.insert(
            "codex".to_string(),
            bootstrap_agent_engine("codex").expect("Codex bootstrap"),
        );
        slots.insert(
            "gemini".to_string(),
            bootstrap_agent_engine("gemini").expect("Gemini bootstrap"),
        );

        let snapshot = discover_provider_sessions_with(
            &slots,
            &ProviderSessionInventorySelector {
                directory_fingerprint: None,
                exact_cwd: Some("E:/Work/BirdCoder".to_string()),
                unique_basename: None,
            },
            |engine_key, _| {
                if engine_key == "gemini" {
                    return Err(sdkwork_agent_kernel::KernelError::provider_error(
                        "gemini_inventory_failure",
                        "fixture failure",
                    ));
                }
                Ok(vec![
                    AgentSession::new("codex-session").with_cwd("E:/Work/BirdCoder")
                ])
            },
            |engine_key, slot| {
                let default_model = slot
                    .list_model_descriptors()
                    .into_iter()
                    .next()
                    .expect("fixture model descriptor");
                Ok((
                    crate::agent_engines::agent_engine_agent_id(engine_key)
                        .expect("fixture agent id")
                        .to_string(),
                    default_model.provider_id,
                    default_model.model_id,
                ))
            },
        )
        .expect("working provider inventory");

        assert_eq!(snapshot.items.len(), 1);
        assert_eq!(snapshot.items[0].engine_key, "codex");
        assert_eq!(snapshot.items[0].session.session_id, "codex-session");
        assert_eq!(snapshot.successful_engine_keys, vec!["codex"]);
        assert_eq!(snapshot.issues.len(), 1);
        assert_eq!(snapshot.issues[0].engine_key, "gemini");
    }

    #[test]
    fn provider_without_projection_identity_is_not_marked_successful() {
        let mut slots = HashMap::new();
        slots.insert(
            "codex".to_string(),
            bootstrap_agent_engine("codex").expect("Codex bootstrap"),
        );

        let snapshot = discover_provider_sessions_with(
            &slots,
            &ProviderSessionInventorySelector {
                directory_fingerprint: None,
                exact_cwd: Some("E:/Work/BirdCoder".to_string()),
                unique_basename: None,
            },
            |_, _| {
                Ok(vec![
                    AgentSession::new("codex-session").with_cwd("E:/Work/BirdCoder")
                ])
            },
            |engine_key, _| {
                Err(format!(
                    "agent engine {engine_key} did not publish a model provider"
                ))
            },
        )
        .expect("degraded provider inventory snapshot");

        assert!(snapshot.items.is_empty());
        assert!(snapshot.successful_engine_keys.is_empty());
        assert_eq!(snapshot.issues.len(), 1);
        assert_eq!(snapshot.issues[0].engine_key, "codex");
        assert!(snapshot.issues[0].reason.contains("model provider"));
    }

    #[test]
    fn empty_provider_inventory_is_successful_when_the_exact_directory_is_known() {
        let mut slots = HashMap::new();
        slots.insert(
            "codex".to_string(),
            bootstrap_agent_engine("codex").expect("Codex bootstrap"),
        );

        let snapshot = discover_provider_sessions_with(
            &slots,
            &ProviderSessionInventorySelector {
                directory_fingerprint: None,
                exact_cwd: Some("E:/Work/BirdCoder".to_string()),
                unique_basename: None,
            },
            |_, _| Ok(Vec::new()),
            |engine_key, slot| {
                let default_model = slot
                    .list_model_descriptors()
                    .into_iter()
                    .next()
                    .expect("fixture model descriptor");
                Ok((
                    crate::agent_engines::agent_engine_agent_id(engine_key)
                        .expect("fixture agent id")
                        .to_string(),
                    default_model.provider_id,
                    default_model.model_id,
                ))
            },
        )
        .expect("empty provider inventory snapshot");

        assert!(snapshot.directory_resolved);
        assert!(snapshot.items.is_empty());
        assert_eq!(snapshot.successful_engine_keys, vec!["codex"]);
        assert!(snapshot.issues.is_empty());
    }

    #[test]
    fn unknown_engine_still_reports_unsupported_engine() {
        let error = load_provider_session_messages(&HashMap::new(), "unknown", "session-1")
            .expect_err("unknown engine must fail");
        assert!(matches!(
            error,
            RuntimeFacadeError::UnsupportedEngine { ref engine_key }
                if engine_key == "unknown"
        ));
    }

    #[test]
    fn directory_fingerprint_selects_one_of_two_same_basename_paths() {
        let fixture_root = create_fingerprint_fixture_root("unique");
        let first = fixture_root.join("first").join("BirdCoder");
        let second = fixture_root.join("second").join("BirdCoder");
        std::fs::create_dir_all(first.join("apps")).expect("create first fixture");
        std::fs::create_dir_all(second.join("packages")).expect("create second fixture");
        let fingerprint =
            provider_session_directory_fingerprint(second.to_str().expect("second fixture path"))
                .expect("fingerprint second fixture");
        let candidates = vec![
            item("codex", "one", first.to_str().expect("first fixture path")),
            item(
                "opencode",
                "two",
                second.to_str().expect("second fixture path"),
            ),
        ];

        let selected = resolve_selected_cwd(
            &candidates,
            &ProviderSessionInventorySelector {
                directory_fingerprint: Some(fingerprint),
                exact_cwd: None,
                unique_basename: Some("BirdCoder".to_string()),
            },
        )
        .expect("select fingerprinted directory");
        let normalized_second =
            normalize_provider_session_path(second.to_str().expect("second fixture path"));
        assert_eq!(selected.as_deref(), Some(normalized_second.as_str()));
        std::fs::remove_dir_all(fixture_root).expect("remove fixtures");
    }

    #[test]
    fn directory_fingerprint_fails_closed_for_identical_roots() {
        let fixture_root = create_fingerprint_fixture_root("ambiguous");
        let first = fixture_root.join("first").join("BirdCoder");
        let second = fixture_root.join("second").join("BirdCoder");
        std::fs::create_dir_all(first.join("apps")).expect("create first fixture");
        std::fs::create_dir_all(second.join("apps")).expect("create second fixture");
        let fingerprint =
            provider_session_directory_fingerprint(first.to_str().expect("first fixture path"))
                .expect("fingerprint first fixture");
        let candidates = vec![
            item("codex", "one", first.to_str().expect("first fixture path")),
            item(
                "opencode",
                "two",
                second.to_str().expect("second fixture path"),
            ),
        ];

        let result = resolve_selected_cwd(
            &candidates,
            &ProviderSessionInventorySelector {
                directory_fingerprint: Some(fingerprint),
                exact_cwd: None,
                unique_basename: Some("BirdCoder".to_string()),
            },
        );
        assert!(matches!(result, Err(RuntimeFacadeError::InvalidInput(_))));
        std::fs::remove_dir_all(fixture_root).expect("remove fixtures");
    }

    #[test]
    fn default_provider_directory_remains_visible() {
        let directory = ProviderSessionDirectoryEntry::default();

        assert!(directory.visible);
        assert!(!directory.pinned);
        assert!(!directory.archived);
    }

    fn create_fingerprint_fixture_root(label: &str) -> std::path::PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("test clock")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "sdkwork-provider-session-selector-{label}-{}-{nonce}",
            std::process::id()
        ))
    }
}
