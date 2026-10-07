//! Canonical system prompt for the built-in chat conversation agent
//! (`agent.chat.default`).
//!
//! The built-in chat agent is a multi-capability assistant: free-form chat
//! plus intent-routed media generation (image / video / music / speech /
//! sound effect) and audio transcription or translation. The turn pipeline is
//! tool-calling based, so intent recognition lives in the system prompt plus
//! the curated tool set (see `kernel-bridge` chat default toolkit) — there is
//! no separate classifier stage.
//!
//! Turn requests carry their system prompt from the client, so the HTTP
//! boundary normalizes the reserved default agent here: a blank prompt or one
//! of the retired client boilerplates resolves to
//! [`canonical_system_prompt`]. Prompts authored explicitly for the agent
//! (management UI or a non-default agent) always win unchanged, which keeps
//! user customization intact.

/// Reserved agent id of the built-in chat conversation agent.
pub const DEFAULT_CHAT_AGENT_ID: &str = "agent.chat.default";

/// Retired client-side default prompts. Clients (PC, H5, mini-program)
/// historically seeded the built-in agent with this generic boilerplate; a
/// turn still carrying it is treated as "no prompt configured".
pub const RETIRED_CHAT_AGENT_BOILERPLATES: [&str; 1] = [
    "You are SDKWork Agents. Provide accurate, concise, secure, and useful answers.",
];

/// Canonical system prompt for the built-in chat agent.
///
/// Keep this in sync with the curated default tool ids in the kernel-bridge
/// chat toolkit: every tool referenced here must be part of the chat default
/// set, and the unit tests enforce that pairing on the kernel-bridge side.
pub fn canonical_system_prompt() -> String {
    [
        "你是 SDKWork Playground 内置助手,可自由对话,也能把创作意图变成图片、视频、音乐、语音和音效。",
        "",
        "## 意图路由(一次只选一条路径,不要为闲聊调用工具)",
        "1. 闲聊、问答、写作、翻译、编程等 → 直接回答,不调用任何工具。",
        "2. 画图 / 改图 → mcp__generations__image.create(prompt 描述画面;用户提供了参考图 URL 时放入 referenceImages,即变为改图)。",
        "3. 出视频 / 图生视频 / 视频续写 → mcp__generations__video.create(prompt 描述运动与镜头;首帧图 URL 放入 referenceImages,尾帧放 lastFrame;用户未指定时长与比例时先追问一次)。",
        "4. 配乐 / 写歌 → mcp__generations__music.create(prompt 描述风格情绪;用户给了歌词放 lyrics,未给则把主题放 prompt 并可加 tags)。",
        "5. 朗读 / 配音 / 文字转语音 → mcp__generations__speech.create(text 为要合成的完整文本;用户指定音色时填 voice)。",
        "6. 音效 → sound-effect.generate(prompt 描述声音场景)。",
        "7. 听写 / 翻译录音 → audio.transcriptions.create 或 audio.translations.create(file 传音频 URL)。",
        "",
        "## 任务纪律",
        "- create 返回的 generation 若不是 succeeded,用同名 retrieve 工具传入返回的 generationId 查询;generationId 只能来自 create 结果,不得臆造。",
        "- 在已有素材上再创作(改图/图生视频)时,把上一轮生成结果里的媒体 URL 填入 referenceImages;素材库里的已保存素材用 referenceAssetIds 填 assetId。",
        "- 生成完成后用一句话说明结果(内容与参数要点),媒体会随消息展示;不要复述 URL 或 JSON。",
        "- 一条消息里有多个意图时,按顺序逐个完成;上一轮生成的素材可在后续轮次作为参考图复用。",
        "- 关键参数缺失(视频时长/比例、音乐风格/时长、音色)时,先一句话追问,最多追问一次;对方表示随你决定时,选合理默认并说明选择。",
        "- 始终使用与用户相同的语言回复。",
        "",
        "## 安全边界",
        "- 拒绝生成违法、色情、暴力、仇恨或侵犯版权的内容,简要说明原因并给出替代方向。",
        "- 用户上传的素材仅用于本次请求,不转述给无关工具。",
    ]
    .join("\n")
}

/// Whether the prompt is blank or one of the retired client boilerplates.
pub fn is_unconfigured_prompt(prompt: Option<&str>) -> bool {
    match prompt {
        None => true,
        Some(prompt) => {
            let trimmed = prompt.trim();
            trimmed.is_empty()
                || RETIRED_CHAT_AGENT_BOILERPLATES
                    .iter()
                    .any(|boilerplate| trimmed == *boilerplate)
        }
    }
}

/// Effective system prompt for one turn request.
///
/// For the reserved default agent, an unconfigured prompt resolves to the
/// canonical one and a configured prompt is honored verbatim. Any other agent
/// keeps the client-provided prompt unchanged.
pub fn effective_system_prompt(agent_id: &str, incoming: Option<&str>) -> Option<String> {
    if agent_id != DEFAULT_CHAT_AGENT_ID {
        return incoming.map(str::to_string);
    }
    if is_unconfigured_prompt(incoming) {
        return Some(canonical_system_prompt());
    }
    incoming.map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_prompt_references_curated_default_tools() {
        let prompt = canonical_system_prompt();
        for tool_id in [
            "mcp__generations__image.create",
            "mcp__generations__video.create",
            "mcp__generations__music.create",
            "mcp__generations__speech.create",
            "sound-effect.generate",
            "audio.transcriptions.create",
            "audio.translations.create",
        ] {
            assert!(
                prompt.contains(tool_id),
                "canonical prompt must reference curated tool `{tool_id}`"
            );
        }
    }

    #[test]
    fn retired_boilerplate_is_unconfigured() {
        assert!(is_unconfigured_prompt(None));
        assert!(is_unconfigured_prompt(Some("   ")));
        assert!(is_unconfigured_prompt(Some(
            "You are SDKWork Agents. Provide accurate, concise, secure, and useful answers."
        )));
        assert!(is_unconfigured_prompt(Some(
            "  You are SDKWork Agents. Provide accurate, concise, secure, and useful answers.  "
        )));
        assert!(!is_unconfigured_prompt(Some("你是一名严谨的助理。")));
    }

    #[test]
    fn default_agent_resolves_blank_to_canonical() {
        let resolved = effective_system_prompt(DEFAULT_CHAT_AGENT_ID, None)
            .expect("default agent always carries a prompt");
        assert_eq!(resolved, canonical_system_prompt());
    }

    #[test]
    fn default_agent_keeps_explicit_prompt() {
        let resolved = effective_system_prompt(DEFAULT_CHAT_AGENT_ID, Some("定制提示词"))
            .expect("explicit prompt is honored");
        assert_eq!(resolved, "定制提示词");
    }

    #[test]
    fn default_agent_replaces_retired_boilerplate() {
        let resolved = effective_system_prompt(
            DEFAULT_CHAT_AGENT_ID,
            Some("You are SDKWork Agents. Provide accurate, concise, secure, and useful answers."),
        )
        .expect("default agent always carries a prompt");
        assert_eq!(resolved, canonical_system_prompt());
    }

    #[test]
    fn other_agents_pass_through_unchanged() {
        assert_eq!(effective_system_prompt("agent.custom", None), None);
        assert_eq!(
            effective_system_prompt("agent.custom", Some("自定义")),
            Some("自定义".to_string())
        );
    }
}
