-- SDKWork official agent catalog live verification.
--
-- Run against the shared development schema after `db:seed`:
--   psql "$SDKWORK_DATABASE_URL" -v ON_ERROR_STOP=1 -f database/verification/agent_catalog_checks.sql
--
-- The script fails closed: any missing binding, dangling reference, or
-- precision-lost policy id raises an exception and leaves a non-zero exit.

BEGIN;

SELECT set_config('app.current_tenant_id', '100001', true);
SELECT set_config('app.current_organization_id', '0', true);

DO $$
DECLARE
    failures integer;
BEGIN
    SELECT count(*) INTO failures
    FROM ai_agent
    WHERE agent_id LIKE 'agent.%'
      AND agent_id <> 'agent.chat.default';
    IF failures <> 64 THEN
        RAISE EXCEPTION 'expected 64 catalog agents, found %', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_agent_version;
    IF failures <> 64 THEN
        RAISE EXCEPTION 'expected 64 agent versions, found %', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_agent_composition_slot;
    IF failures <> 364 THEN
        RAISE EXCEPTION 'expected 364 composition slots, found %', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_agent a
    WHERE a.agent_id LIKE 'agent.%'
      AND a.agent_id <> 'agent.chat.default'
      AND NOT EXISTS (
          SELECT 1 FROM ai_agent_composition_slot s
          WHERE s.tenant_id = a.tenant_id AND s.agent_id = a.agent_id
            AND s.slot_kind = 'knowledge' AND s.enabled
      );
    IF failures <> 0 THEN
        RAISE EXCEPTION '% agents lack an enabled knowledge binding', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_agent a
    WHERE a.agent_id LIKE 'agent.%'
      AND a.agent_id <> 'agent.chat.default'
      AND NOT EXISTS (
          SELECT 1 FROM ai_agent_composition_slot s
          WHERE s.tenant_id = a.tenant_id AND s.agent_id = a.agent_id
            AND s.slot_kind = 'memory' AND s.enabled
      );
    IF failures <> 0 THEN
        RAISE EXCEPTION '% agents lack an enabled memory binding', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_agent a
    WHERE a.agent_id LIKE 'agent.%'
      AND a.agent_id <> 'agent.chat.default'
      AND NOT EXISTS (
          SELECT 1 FROM ai_agent_composition_slot s
          WHERE s.tenant_id = a.tenant_id AND s.agent_id = a.agent_id
            AND s.slot_kind = 'mcp' AND s.enabled
      );
    IF failures <> 0 THEN
        RAISE EXCEPTION '% agents lack an enabled MCP binding', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_agent a
    WHERE a.agent_id LIKE 'agent.%'
      AND a.agent_id <> 'agent.chat.default'
      AND NOT EXISTS (
          SELECT 1 FROM ai_agent_composition_slot s
          WHERE s.tenant_id = a.tenant_id AND s.agent_id = a.agent_id
            AND s.slot_kind = 'skill' AND s.enabled
      );
    IF failures <> 0 THEN
        RAISE EXCEPTION '% agents lack an enabled skill binding', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_agent a
    JOIN ai_agent_composition_slot s
      ON s.tenant_id = a.tenant_id AND s.agent_id = a.agent_id AND s.slot_kind = 'knowledge'
    LEFT JOIN kb_space k
      ON k.id = s.target_ref::bigint AND k.tenant_id = a.tenant_id
    WHERE a.agent_id LIKE 'agent.%'
      AND a.agent_id <> 'agent.chat.default'
      AND k.id IS NULL;
    IF failures <> 0 THEN
        RAISE EXCEPTION '% knowledge slots reference a missing kb_space', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_agent a
    JOIN ai_agent_composition_slot s
      ON s.tenant_id = a.tenant_id AND s.agent_id = a.agent_id AND s.slot_kind = 'memory'
    LEFT JOIN ai_space m
      ON m.id = replace(s.target_ref, 'space:', '')::bigint AND m.tenant_id = a.tenant_id
    WHERE a.agent_id LIKE 'agent.%'
      AND a.agent_id <> 'agent.chat.default'
      AND m.id IS NULL;
    IF failures <> 0 THEN
        RAISE EXCEPTION '% memory slots reference a missing ai_space', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_agent a
    JOIN ai_agent_composition_slot s
      ON s.tenant_id = a.tenant_id AND s.agent_id = a.agent_id AND s.slot_kind = 'mcp'
    LEFT JOIN ai_mcp_server m
      ON m.server_key = s.target_ref AND m.tenant_id = a.tenant_id
    WHERE a.agent_id LIKE 'agent.%'
      AND a.agent_id <> 'agent.chat.default'
      AND m.server_key IS NULL;
    IF failures <> 0 THEN
        RAISE EXCEPTION '% MCP slots reference a missing ai_mcp_server', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_agent_composition_slot
    WHERE slot_kind = 'knowledge'
      AND policy_json->>'spaceId' <> target_ref;
    IF failures <> 0 THEN
        RAISE EXCEPTION '% knowledge slots have a policy/target id mismatch', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_agent
    WHERE agent_id LIKE 'agent.%'
      AND agent_id <> 'agent.chat.default'
      AND coalesce(default_code_task_intent_json->'constraints', '[]'::jsonb)::text NOT LIKE '%knowledge.base.%';
    IF failures <> 0 THEN
        RAISE EXCEPTION '% agents lack a management-profile knowledge reference', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM (
        SELECT tenant_id, agent_id, slot_id
        FROM ai_agent_composition_slot
        GROUP BY 1, 2, 3
        HAVING count(*) > 1
    ) duplicates;
    IF failures <> 0 THEN
        RAISE EXCEPTION '% duplicate composition slot ids', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM kb_space
    WHERE id BETWEEN 533000000000000001 AND 533000000000000064;
    IF failures <> 64 THEN
        RAISE EXCEPTION 'expected 64 knowledge spaces, found %', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_space
    WHERE id BETWEEN 534000000000000001 AND 534000000000000064;
    IF failures <> 64 THEN
        RAISE EXCEPTION 'expected 64 memory spaces, found %', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_subject
    WHERE id BETWEEN 536000000000000001 AND 536000000000000064;
    IF failures <> 64 THEN
        RAISE EXCEPTION 'expected 64 memory subjects, found %', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_memory_binding
    WHERE id BETWEEN 537000000000000001 AND 537000000000000064;
    IF failures <> 64 THEN
        RAISE EXCEPTION 'expected 64 memory bindings, found %', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_memory_binding
    WHERE id BETWEEN 537000000000000001 AND 537000000000000064
      AND (binding_kind <> 'agent_space' OR binding_role <> 'retriever');
    IF failures <> 0 THEN
        RAISE EXCEPTION '% memory bindings use an invalid kind or role', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_mcp_server
    WHERE server_key LIKE 'mcp.sdkwork.%';
    IF failures < 18 THEN
        RAISE EXCEPTION 'expected at least 18 MCP catalog entries, found %', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_mcp_server_category
    WHERE id BETWEEN 539000000000000001 AND 539000000000000012;
    IF failures <> 12 THEN
        RAISE EXCEPTION 'expected 12 MCP category entries, found %', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_mcp_connector
    WHERE id BETWEEN 538000000000000001 AND 538000000000000018;
    IF failures <> 18 THEN
        RAISE EXCEPTION 'expected 18 draft MCP connectors, found %', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_mcp_server
    WHERE server_key LIKE 'mcp.sdkwork.%'
      AND latest_connector_id IS NULL;
    IF failures <> 0 THEN
        RAISE EXCEPTION '% MCP servers have no latest connector', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_agent_composition_slot
    WHERE slot_kind = 'mcp'
      AND (
          policy_json->>'connectorRef' IS NULL
          OR policy_json->>'activationState' <> 'catalog_draft'
      );
    IF failures <> 0 THEN
        RAISE EXCEPTION '% MCP slots lack draft-connector activation metadata', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_agent
    WHERE agent_id LIKE 'agent.%'
      AND agent_id <> 'agent.chat.default'
      AND length(default_code_task_intent_json->>'prompt') < 900;
    IF failures <> 0 THEN
        RAISE EXCEPTION '% agents have a prompt shorter than 900 characters', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_agent_version v
    JOIN ai_agent a
      ON a.tenant_id = v.tenant_id AND a.agent_id = v.agent_id
    WHERE a.agent_id LIKE 'agent.%'
      AND a.agent_id <> 'agent.chat.default'
      AND v.manifest_json->>'display_name' <> a.display_name;
    IF failures <> 0 THEN
        RAISE EXCEPTION '% agent versions are not localized with the live agent', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_agent
    WHERE agent_id LIKE 'agent.%'
      AND agent_id <> 'agent.chat.default'
      AND manifest_json->>'domain' NOT IN (
          'engineering', 'education', 'healthcare', 'legal', 'travel', 'ecommerce', 'realestate',
          'restaurant', 'enterprise', 'government', 'hospitality', 'writing'
      );
    IF failures <> 0 THEN
        RAISE EXCEPTION '% agents use an unknown category', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_agent
    WHERE agent_id LIKE 'agent.%'
      AND agent_id <> 'agent.chat.default'
      AND manifest_json->>'domain' = 'writing';
    IF failures <> 6 THEN
        RAISE EXCEPTION 'expected 6 writing agents, found %', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_agent a
    WHERE a.agent_id LIKE 'agent.%'
      AND a.agent_id <> 'agent.chat.default'
      AND a.manifest_json->>'domain' = 'writing'
      AND NOT EXISTS (
          SELECT 1 FROM ai_agent_composition_slot s
          WHERE s.tenant_id = a.tenant_id AND s.agent_id = a.agent_id
            AND s.slot_kind = 'mcp'
            AND s.target_ref = 'mcp.sdkwork.writing-sources'
      );
    IF failures <> 0 THEN
        RAISE EXCEPTION '% writing agents lack the writing-sources MCP binding', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_agent_composition_slot
    WHERE slot_kind = 'mcp'
      AND target_ref = 'mcp.sdkwork.knowledgebase-notes'
      AND policy_json->>'toolPolicy' = 'read-write'
      AND jsonb_array_length(policy_json->'tools') >= 5;
    IF failures <> 4 THEN
        RAISE EXCEPTION 'expected 4 read-write knowledgebase-notes slots, found %', failures;
    END IF;

    RAISE NOTICE 'agent catalog verification passed';
END $$;

SELECT
    (SELECT count(*) FROM ai_agent WHERE agent_id LIKE 'agent.%' AND agent_id <> 'agent.chat.default') AS agents,
    (SELECT count(*) FROM ai_agent_version) AS versions,
    (SELECT count(*) FROM ai_agent_composition_slot) AS slots,
    (SELECT count(*) FROM kb_space WHERE id BETWEEN 533000000000000001 AND 533000000000000064) AS knowledge_spaces,
    (SELECT count(*) FROM ai_space WHERE id BETWEEN 534000000000000001 AND 534000000000000064) AS memory_spaces,
    (SELECT count(*) FROM ai_mcp_server WHERE server_key LIKE 'mcp.sdkwork.%') AS mcp_servers,
    (SELECT count(*) FROM ai_mcp_server_category WHERE id BETWEEN 539000000000000001 AND 539000000000000012) AS mcp_categories,
    (SELECT count(*) FROM ai_mcp_connector WHERE id BETWEEN 538000000000000001 AND 538000000000000018) AS mcp_connectors;

COMMIT;
