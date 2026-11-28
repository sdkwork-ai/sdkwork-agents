-- SDKWork expert catalog live verification.
--
-- Run against the shared development schema after `db:seed`:
--   psql "$SDKWORK_DATABASE_URL" -v ON_ERROR_STOP=1 -f database/verification/expert_catalog_checks.sql
--
-- The script fails closed: any missing expert, team, member, persona, market
-- profile or collection, or a dangling reference, raises an exception and
-- leaves a non-zero exit.

BEGIN;

SELECT set_config('app.current_tenant_id', '100001', true);
SELECT set_config('app.current_organization_id', '0', true);

DO $$
DECLARE
    failures integer;
    locale_count integer;
BEGIN
    -- Scope every assertion to the seeded expert catalog. A tenant may own
    -- custom agents; those are not part of the catalog contract and must not
    -- change the expected counts.
    CREATE TEMP TABLE expert_agent_ids ON COMMIT DROP AS
        SELECT id, tenant_id, organization_id, agent_id
        FROM ai_agent
        WHERE manifest_json->>'owner_name' = 'SDKWork'
          AND manifest_json->>'manifest_type' = 'agent'
          AND manifest_json->>'domain' = 'experts';

    SELECT count(*) INTO failures FROM expert_agent_ids;
    IF failures <> 331 THEN
        RAISE EXCEPTION 'expected 331 expert agents, found %', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_agent_version v
    JOIN expert_agent_ids c
      ON c.tenant_id = v.tenant_id AND c.agent_id = v.agent_id;
    IF failures <> 331 THEN
        RAISE EXCEPTION 'expected 331 expert agent versions, found %', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_agent_category
    WHERE tenant_id = 100001 AND organization_id = 0
      AND category_code LIKE 'expert-%'
      AND deleted_at IS NULL;
    IF failures <> 22 THEN
        RAISE EXCEPTION 'expected 22 expert categories, found %', failures;
    END IF;

    -- Every expert belongs to a seeded expert category.
    SELECT count(*) INTO failures
    FROM expert_agent_ids c
    JOIN ai_agent a ON a.tenant_id = c.tenant_id AND a.agent_id = c.agent_id
    LEFT JOIN ai_agent_category cat
      ON cat.tenant_id = a.tenant_id
     AND cat.organization_id = a.organization_id
     AND cat.category_code = a.category_code
     AND cat.deleted_at IS NULL
    WHERE a.category_code IS NULL OR cat.id IS NULL;
    IF failures <> 0 THEN
        RAISE EXCEPTION '% expert agents have no seeded category', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_expert_team
    WHERE tenant_id = 100001 AND organization_id = 0 AND deleted_at IS NULL;
    IF failures <> 5 THEN
        RAISE EXCEPTION 'expected 5 expert teams, found %', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_expert_team_member
    WHERE tenant_id = 100001 AND organization_id = 0 AND deleted_at IS NULL;
    IF failures <> 15 THEN
        RAISE EXCEPTION 'expected 15 expert team members, found %', failures;
    END IF;

    -- Every team member resolves to a seeded expert agent.
    SELECT count(*) INTO failures
    FROM ai_expert_team_member m
    LEFT JOIN expert_agent_ids c
      ON c.tenant_id = m.tenant_id AND c.agent_id = m.expert_agent_id
    WHERE m.tenant_id = 100001 AND m.organization_id = 0
      AND m.deleted_at IS NULL
      AND c.agent_id IS NULL;
    IF failures <> 0 THEN
        RAISE EXCEPTION '% team members reference an unseeded expert', failures;
    END IF;

    -- Every team carries 2-8 ordered members.
    SELECT count(*) INTO failures
    FROM ai_expert_team t
    WHERE t.tenant_id = 100001 AND t.organization_id = 0 AND t.deleted_at IS NULL
      AND (
          SELECT count(*) FROM ai_expert_team_member m
          WHERE m.tenant_id = t.tenant_id
            AND m.organization_id = t.organization_id
            AND m.team_id = t.team_id
            AND m.deleted_at IS NULL
      ) NOT BETWEEN 2 AND 8;
    IF failures <> 0 THEN
        RAISE EXCEPTION '% expert teams have an out-of-range member count', failures;
    END IF;

    -- Every seeded team must be ingestible by the expert package that summons
    -- it: that contract caps the shared constraints at 2000 characters and
    -- requires a prompt for a custom coordinator. Storage enforces both; the
    -- check keeps a future baseline change from silently relaxing them.
    SELECT count(*) INTO failures
    FROM ai_expert_team
    WHERE tenant_id = 100001 AND organization_id = 0 AND deleted_at IS NULL
      AND (
          char_length(constraints_text) > 2000
          OR (coordinator_mode = 'custom' AND char_length(BTRIM(coordinator_prompt)) = 0)
      );
    IF failures <> 0 THEN
        RAISE EXCEPTION '% expert teams are outside the summonable contract (constraints > 2000 chars, or a custom coordinator without a prompt)', failures;
    END IF;

    -- Personas are seeded per active deployment locale: the seed profile
    -- applies the locale sets the deployment configures, so a single-locale
    -- deployment carries one persona per expert. The catalog must still be
    -- complete in every locale it did seed, and zh-CN is the required one.
    SELECT count(DISTINCT p.locale) INTO locale_count
    FROM ai_expert_persona p
    WHERE p.tenant_id = 100001 AND p.organization_id = 0;
    IF NOT EXISTS (
        SELECT 1 FROM ai_expert_persona p
        WHERE p.tenant_id = 100001 AND p.organization_id = 0 AND p.locale = 'zh-CN'
    ) THEN
        RAISE EXCEPTION 'expected a zh-CN expert persona set';
    END IF;

    SELECT count(*) INTO failures
    FROM ai_expert_persona
    WHERE tenant_id = 100001 AND organization_id = 0;
    IF failures <> 331 * locale_count THEN
        RAISE EXCEPTION 'expected 331 personas per seeded locale (% locales), found %',
            locale_count, failures;
    END IF;

    -- Every expert carries every seeded locale.
    SELECT count(*) INTO failures
    FROM expert_agent_ids c
    WHERE (
        SELECT count(DISTINCT p.locale)
        FROM ai_expert_persona p
        WHERE p.tenant_id = c.tenant_id
          AND p.organization_id = c.organization_id
          AND p.expert_agent_id = c.agent_id
    ) <> locale_count;
    IF failures <> 0 THEN
        RAISE EXCEPTION '% expert agents are missing a seeded locale persona', failures;
    END IF;

    -- Persona rows never belong to an agent outside the expert catalog.
    SELECT count(*) INTO failures
    FROM ai_expert_persona p
    LEFT JOIN expert_agent_ids c
      ON c.tenant_id = p.tenant_id AND c.agent_id = p.expert_agent_id
    WHERE c.agent_id IS NULL;
    IF failures <> 0 THEN
        RAISE EXCEPTION '% personas reference a non-expert agent', failures;
    END IF;

    -- Market layer: exactly one editorial profile per seeded expert.
    SELECT count(*) INTO failures
    FROM ai_expert_profile
    WHERE tenant_id = 100001 AND organization_id = 0 AND deleted_at IS NULL;
    IF failures <> 331 THEN
        RAISE EXCEPTION 'expected 331 expert market profiles, found %', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM expert_agent_ids c
    WHERE NOT EXISTS (
        SELECT 1 FROM ai_expert_profile p
        WHERE p.tenant_id = c.tenant_id
          AND p.organization_id = c.organization_id
          AND p.expert_agent_id = c.agent_id
          AND p.deleted_at IS NULL
    );
    IF failures <> 0 THEN
        RAISE EXCEPTION '% expert agents are missing a market profile', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_expert_profile
    WHERE tenant_id = 100001 AND organization_id = 0 AND deleted_at IS NULL
      AND (usage_count < 0 OR sort_order < 0);
    IF failures <> 0 THEN
        RAISE EXCEPTION '% expert profiles carry a negative usage count or sort order', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_expert_collection
    WHERE tenant_id = 100001 AND organization_id = 0 AND deleted_at IS NULL;
    IF failures <> 6 THEN
        RAISE EXCEPTION 'expected 6 expert collections, found %', failures;
    END IF;

    -- Every collection's cover key is the stable code suffix.
    SELECT count(*) INTO failures
    FROM ai_expert_collection
    WHERE tenant_id = 100001 AND organization_id = 0 AND deleted_at IS NULL
      AND cover_key <> replace(collection_code, 'collection-', '');
    IF failures <> 0 THEN
        RAISE EXCEPTION '% expert collections carry a cover key that is not the code suffix', failures;
    END IF;

    SELECT count(*) INTO failures
    FROM ai_expert_collection_item
    WHERE tenant_id = 100001 AND organization_id = 0 AND deleted_at IS NULL;
    IF failures <> 18 THEN
        RAISE EXCEPTION 'expected 18 expert collection items, found %', failures;
    END IF;

    -- Every collection member resolves to a seeded expert and its own shelf.
    SELECT count(*) INTO failures
    FROM ai_expert_collection_item i
    LEFT JOIN expert_agent_ids c
      ON c.tenant_id = i.tenant_id AND c.agent_id = i.expert_agent_id
    LEFT JOIN ai_expert_collection col
      ON col.tenant_id = i.tenant_id
     AND col.organization_id = i.organization_id
     AND col.collection_code = i.collection_code
     AND col.deleted_at IS NULL
    WHERE i.tenant_id = 100001 AND i.organization_id = 0
      AND i.deleted_at IS NULL
      AND (c.agent_id IS NULL OR col.id IS NULL);
    IF failures <> 0 THEN
        RAISE EXCEPTION '% collection items reference an unseeded expert or collection', failures;
    END IF;

    RAISE NOTICE 'expert catalog verified: 331 agents, 22 categories, 5 teams, 15 members, 662 personas, 331 profiles, 6 collections, 18 items';
END
$$;

COMMIT;
