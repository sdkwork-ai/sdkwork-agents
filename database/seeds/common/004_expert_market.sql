-- SDKWork expert market: the curated expert collections and the
-- per-expert editorial profile (usage count, badge, catalog order,
-- publish time). Hand-maintained: the catalog rows themselves are
-- generated into 003_expert_catalog.sql.

-- Deterministic id bases: profile 373000000000000001, collection
-- 374000000000000001, collection item 375000000000000001.

BEGIN;

INSERT INTO ai_expert_collection (
  id, uuid, tenant_id, organization_id, collection_code, name, name_en,
  description, description_en, cover_key, status, sort_order, version,
  created_by, updated_by, created_at, updated_at, deleted_at, deleted_by
) VALUES (
  374000000000000001, '7a9dbcc4-6e3e-426f-8bd3-1b77250857b1', 100001, 0,
  'collection-ecommerce', '电商运营', 'E-commerce Operations',
  '精选电商运营专家，覆盖店铺增长、直播带货与跨境履约。', 'Curated e-commerce operators for storefront growth, livestream commerce and cross-border fulfillment.', 'ecommerce',
  1, 10, 0, 1, 1,
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code) DO UPDATE SET
  name = EXCLUDED.name,
  name_en = EXCLUDED.name_en,
  description = EXCLUDED.description,
  description_en = EXCLUDED.description_en,
  cover_key = EXCLUDED.cover_key,
  status = EXCLUDED.status,
  sort_order = EXCLUDED.sort_order,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection_item (
  id, uuid, tenant_id, organization_id, collection_code, expert_agent_id,
  position, label, label_en, created_at, updated_at, deleted_at
) VALUES (
  375000000000000001, 'c7edbf6c-9db9-4992-8b68-b7f9ec3b99a2', 100001, 0,
  'collection-ecommerce', 'agent.expert.marketing-ecommerce-operator', 0,
  '电商运营专家', 'E-commerce Operations Specialist',
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code, expert_agent_id) DO UPDATE SET
  position = EXCLUDED.position,
  label = EXCLUDED.label,
  label_en = EXCLUDED.label_en,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection_item (
  id, uuid, tenant_id, organization_id, collection_code, expert_agent_id,
  position, label, label_en, created_at, updated_at, deleted_at
) VALUES (
  375000000000000002, 'f00a2193-026d-44d4-84bd-54ee43cd8e3a', 100001, 0,
  'collection-ecommerce', 'agent.expert.marketing-china-ecommerce-operator', 1,
  '国内电商操盘手', 'China E-Commerce Operator',
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code, expert_agent_id) DO UPDATE SET
  position = EXCLUDED.position,
  label = EXCLUDED.label,
  label_en = EXCLUDED.label_en,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection_item (
  id, uuid, tenant_id, organization_id, collection_code, expert_agent_id,
  position, label, label_en, created_at, updated_at, deleted_at
) VALUES (
  375000000000000003, 'e156bcbf-e4cd-473a-80c5-286787d3a4d0', 100001, 0,
  'collection-ecommerce', 'agent.expert.marketing-cross-border-ecommerce', 2,
  '跨境电商顾问', 'Cross-Border E-Commerce Specialist',
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code, expert_agent_id) DO UPDATE SET
  position = EXCLUDED.position,
  label = EXCLUDED.label,
  label_en = EXCLUDED.label_en,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection (
  id, uuid, tenant_id, organization_id, collection_code, name, name_en,
  description, description_en, cover_key, status, sort_order, version,
  created_by, updated_by, created_at, updated_at, deleted_at, deleted_by
) VALUES (
  374000000000000002, 'f057eec0-ebe8-403a-8726-6b3b11013799', 100001, 0,
  'collection-content', '内容创作', 'Content Creation',
  '内容策划、长篇写作与多平台分发的常驻组合。', 'A resident lineup for content planning, long-form writing and multi-platform distribution.', 'content',
  1, 20, 0, 1, 1,
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code) DO UPDATE SET
  name = EXCLUDED.name,
  name_en = EXCLUDED.name_en,
  description = EXCLUDED.description,
  description_en = EXCLUDED.description_en,
  cover_key = EXCLUDED.cover_key,
  status = EXCLUDED.status,
  sort_order = EXCLUDED.sort_order,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection_item (
  id, uuid, tenant_id, organization_id, collection_code, expert_agent_id,
  position, label, label_en, created_at, updated_at, deleted_at
) VALUES (
  375000000000000011, '19224eef-4a09-4d0a-8406-b8ced5043a90', 100001, 0,
  'collection-content', 'agent.expert.marketing-content-creator', 0,
  '内容创作者', 'Content Creator',
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code, expert_agent_id) DO UPDATE SET
  position = EXCLUDED.position,
  label = EXCLUDED.label,
  label_en = EXCLUDED.label_en,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection_item (
  id, uuid, tenant_id, organization_id, collection_code, expert_agent_id,
  position, label, label_en, created_at, updated_at, deleted_at
) VALUES (
  375000000000000012, '3c6ce10b-fdde-4c28-87e0-ba42f7125702', 100001, 0,
  'collection-content', 'agent.expert.marketing-book-co-author', 1,
  '图书合著者', 'Book Co-Author',
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code, expert_agent_id) DO UPDATE SET
  position = EXCLUDED.position,
  label = EXCLUDED.label,
  label_en = EXCLUDED.label_en,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection_item (
  id, uuid, tenant_id, organization_id, collection_code, expert_agent_id,
  position, label, label_en, created_at, updated_at, deleted_at
) VALUES (
  375000000000000013, 'b893999f-f9fb-4346-85b1-6c963d206483', 100001, 0,
  'collection-content', 'agent.expert.marketing-multi-platform-publisher', 2,
  '多平台发布官', 'Multi-Platform Publisher',
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code, expert_agent_id) DO UPDATE SET
  position = EXCLUDED.position,
  label = EXCLUDED.label,
  label_en = EXCLUDED.label_en,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection (
  id, uuid, tenant_id, organization_id, collection_code, name, name_en,
  description, description_en, cover_key, status, sort_order, version,
  created_by, updated_by, created_at, updated_at, deleted_at, deleted_by
) VALUES (
  374000000000000003, 'f26a5840-aad9-43ab-85d5-ab8e2484b90c', 100001, 0,
  'collection-investment', '投资分析', 'Investment Analysis',
  '从行业研究到财务建模与预测的投资分析小组。', 'An investment analysis pod spanning industry research, financial modeling and forecasting.', 'investment',
  1, 30, 0, 1, 1,
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code) DO UPDATE SET
  name = EXCLUDED.name,
  name_en = EXCLUDED.name_en,
  description = EXCLUDED.description,
  description_en = EXCLUDED.description_en,
  cover_key = EXCLUDED.cover_key,
  status = EXCLUDED.status,
  sort_order = EXCLUDED.sort_order,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection_item (
  id, uuid, tenant_id, organization_id, collection_code, expert_agent_id,
  position, label, label_en, created_at, updated_at, deleted_at
) VALUES (
  375000000000000021, 'a7464286-648b-4eb8-89d7-4851e104661c', 100001, 0,
  'collection-investment', 'agent.expert.finance-investment-researcher', 0,
  '投资研究员', 'Investment Researcher',
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code, expert_agent_id) DO UPDATE SET
  position = EXCLUDED.position,
  label = EXCLUDED.label,
  label_en = EXCLUDED.label_en,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection_item (
  id, uuid, tenant_id, organization_id, collection_code, expert_agent_id,
  position, label, label_en, created_at, updated_at, deleted_at
) VALUES (
  375000000000000022, 'e96fd12e-e838-42cc-86f3-f0f8d3b1fc01', 100001, 0,
  'collection-investment', 'agent.expert.finance-financial-analyst', 1,
  '财务分析师', 'Financial Analyst',
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code, expert_agent_id) DO UPDATE SET
  position = EXCLUDED.position,
  label = EXCLUDED.label,
  label_en = EXCLUDED.label_en,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection_item (
  id, uuid, tenant_id, organization_id, collection_code, expert_agent_id,
  position, label, label_en, created_at, updated_at, deleted_at
) VALUES (
  375000000000000023, '6a18ddfb-c802-4032-824e-2a7b568aa8a4', 100001, 0,
  'collection-investment', 'agent.expert.finance-financial-forecaster', 2,
  '财务预测分析师', 'Financial Forecasting Analyst',
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code, expert_agent_id) DO UPDATE SET
  position = EXCLUDED.position,
  label = EXCLUDED.label,
  label_en = EXCLUDED.label_en,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection (
  id, uuid, tenant_id, organization_id, collection_code, name, name_en,
  description, description_en, cover_key, status, sort_order, version,
  created_by, updated_by, created_at, updated_at, deleted_at, deleted_by
) VALUES (
  374000000000000004, '1e354299-772d-4713-8ba8-f4970d4e5968', 100001, 0,
  'collection-legal', '法律咨询', 'Legal Advisory',
  '合同审查、政策起草与合规核查的法律支持组合。', 'Legal support for contract review, policy drafting and compliance checks.', 'legal',
  1, 40, 0, 1, 1,
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code) DO UPDATE SET
  name = EXCLUDED.name,
  name_en = EXCLUDED.name_en,
  description = EXCLUDED.description,
  description_en = EXCLUDED.description_en,
  cover_key = EXCLUDED.cover_key,
  status = EXCLUDED.status,
  sort_order = EXCLUDED.sort_order,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection_item (
  id, uuid, tenant_id, organization_id, collection_code, expert_agent_id,
  position, label, label_en, created_at, updated_at, deleted_at
) VALUES (
  375000000000000031, '351218ca-022f-4a77-8f18-d91a5d24d8f8', 100001, 0,
  'collection-legal', 'agent.expert.legal-contract-reviewer', 0,
  '合同审查专家', 'Contract Review Specialist',
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code, expert_agent_id) DO UPDATE SET
  position = EXCLUDED.position,
  label = EXCLUDED.label,
  label_en = EXCLUDED.label_en,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection_item (
  id, uuid, tenant_id, organization_id, collection_code, expert_agent_id,
  position, label, label_en, created_at, updated_at, deleted_at
) VALUES (
  375000000000000032, 'c0e94d46-0624-456d-8b8e-e32cd0781262', 100001, 0,
  'collection-legal', 'agent.expert.legal-policy-writer', 1,
  '法律政策撰写人', 'Legal Policy Writer',
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code, expert_agent_id) DO UPDATE SET
  position = EXCLUDED.position,
  label = EXCLUDED.label,
  label_en = EXCLUDED.label_en,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection_item (
  id, uuid, tenant_id, organization_id, collection_code, expert_agent_id,
  position, label, label_en, created_at, updated_at, deleted_at
) VALUES (
  375000000000000033, '148dcaca-1d54-4468-8fc0-35a22517a406', 100001, 0,
  'collection-legal', 'agent.expert.support-legal-compliance-checker', 2,
  '合规核查员', 'Legal Compliance Checker',
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code, expert_agent_id) DO UPDATE SET
  position = EXCLUDED.position,
  label = EXCLUDED.label,
  label_en = EXCLUDED.label_en,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection (
  id, uuid, tenant_id, organization_id, collection_code, name, name_en,
  description, description_en, cover_key, status, sort_order, version,
  created_by, updated_by, created_at, updated_at, deleted_at, deleted_by
) VALUES (
  374000000000000005, '311e95ed-ab63-4fc0-8df7-7628be120c86', 100001, 0,
  'collection-smallbusiness', '小微企业', 'Small Business',
  '为小微企业主准备的经营、财务与运营顾问。', 'Advisors for small-business owners covering strategy, bookkeeping and operations.', 'smallbusiness',
  1, 50, 0, 1, 1,
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code) DO UPDATE SET
  name = EXCLUDED.name,
  name_en = EXCLUDED.name_en,
  description = EXCLUDED.description,
  description_en = EXCLUDED.description_en,
  cover_key = EXCLUDED.cover_key,
  status = EXCLUDED.status,
  sort_order = EXCLUDED.sort_order,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection_item (
  id, uuid, tenant_id, organization_id, collection_code, expert_agent_id,
  position, label, label_en, created_at, updated_at, deleted_at
) VALUES (
  375000000000000041, 'f66dab86-efc0-40fe-886d-10e25b544729', 100001, 0,
  'collection-smallbusiness', 'agent.expert.business-strategist', 0,
  '商业策略顾问', 'Business Strategist',
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code, expert_agent_id) DO UPDATE SET
  position = EXCLUDED.position,
  label = EXCLUDED.label,
  label_en = EXCLUDED.label_en,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection_item (
  id, uuid, tenant_id, organization_id, collection_code, expert_agent_id,
  position, label, label_en, created_at, updated_at, deleted_at
) VALUES (
  375000000000000042, 'a12cb285-0156-4577-84d5-133f9268a736', 100001, 0,
  'collection-smallbusiness', 'agent.expert.finance-bookkeeper-controller', 1,
  '记账与财务主管', 'Bookkeeper & Controller',
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code, expert_agent_id) DO UPDATE SET
  position = EXCLUDED.position,
  label = EXCLUDED.label,
  label_en = EXCLUDED.label_en,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection_item (
  id, uuid, tenant_id, organization_id, collection_code, expert_agent_id,
  position, label, label_en, created_at, updated_at, deleted_at
) VALUES (
  375000000000000043, 'f1eabeab-41b7-412d-8f76-b27c3a815e77', 100001, 0,
  'collection-smallbusiness', 'agent.expert.operations-manager', 2,
  '运营经理', 'Operations Manager',
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code, expert_agent_id) DO UPDATE SET
  position = EXCLUDED.position,
  label = EXCLUDED.label,
  label_en = EXCLUDED.label_en,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection (
  id, uuid, tenant_id, organization_id, collection_code, name, name_en,
  description, description_en, cover_key, status, sort_order, version,
  created_by, updated_by, created_at, updated_at, deleted_at, deleted_by
) VALUES (
  374000000000000006, '88c6ecb9-fe5c-4334-8442-9a04f1795074', 100001, 0,
  'collection-data', '数据分析', 'Data Analysis',
  '数据汇总、分析报告与研究综合的数据小组。', 'A data pod for consolidation, analytics reporting and research synthesis.', 'data',
  1, 60, 0, 1, 1,
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code) DO UPDATE SET
  name = EXCLUDED.name,
  name_en = EXCLUDED.name_en,
  description = EXCLUDED.description,
  description_en = EXCLUDED.description_en,
  cover_key = EXCLUDED.cover_key,
  status = EXCLUDED.status,
  sort_order = EXCLUDED.sort_order,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection_item (
  id, uuid, tenant_id, organization_id, collection_code, expert_agent_id,
  position, label, label_en, created_at, updated_at, deleted_at
) VALUES (
  375000000000000051, 'e51493f8-8a09-4615-8f58-a8edd7fe51de', 100001, 0,
  'collection-data', 'agent.expert.support-analytics-reporter', 0,
  '分析报告专员', 'Analytics Reporter',
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code, expert_agent_id) DO UPDATE SET
  position = EXCLUDED.position,
  label = EXCLUDED.label,
  label_en = EXCLUDED.label_en,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection_item (
  id, uuid, tenant_id, organization_id, collection_code, expert_agent_id,
  position, label, label_en, created_at, updated_at, deleted_at
) VALUES (
  375000000000000052, '7d54aadf-c390-4166-8be1-f8f8624aab38', 100001, 0,
  'collection-data', 'agent.expert.data-consolidation-agent', 1,
  '数据汇总专员', 'Data Consolidation Agent',
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code, expert_agent_id) DO UPDATE SET
  position = EXCLUDED.position,
  label = EXCLUDED.label,
  label_en = EXCLUDED.label_en,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_collection_item (
  id, uuid, tenant_id, organization_id, collection_code, expert_agent_id,
  position, label, label_en, created_at, updated_at, deleted_at
) VALUES (
  375000000000000053, '43af28ad-009f-4b63-8b0f-71de0d6cdef8', 100001, 0,
  'collection-data', 'agent.expert.research-synthesist', 2,
  '研究综合分析师', 'Research Synthesist',
  TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL
)
ON CONFLICT (tenant_id, organization_id, collection_code, expert_agent_id) DO UPDATE SET
  position = EXCLUDED.position,
  label = EXCLUDED.label,
  label_en = EXCLUDED.label_en,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000001, 'f2971a59-7745-445e-8eca-828ba786a987', 100001, 0,
  'agent.expert.academic-anthropologist', 25183,
  NULL, 0, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000002, 'cce78d99-4cc1-4e18-8993-574f52035238', 100001, 0,
  'agent.expert.academic-geographer', 23447,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000003, '73cf35cc-3edd-443d-840e-58d10cdf21cb', 100001, 0,
  'agent.expert.academic-historian', 34796,
  NULL, 2, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000004, '9c946b03-5227-4b98-8d53-76b23776200f', 100001, 0,
  'agent.expert.academic-narratologist', 41329,
  NULL, 3, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000005, 'ad5d8a79-9104-4a1e-807e-ea84ef11d236', 100001, 0,
  'agent.expert.academic-psychologist', 24501,
  NULL, 4, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000006, 'ba84f585-8ebc-427c-87ac-d1ae4f395e0d', 100001, 0,
  'agent.expert.academic-statistician', 4776,
  NULL, 5, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000007, '9f3d3cbe-96af-4c62-846f-19f8a2e64187', 100001, 0,
  'agent.expert.academic-study-planner', 28915,
  NULL, 6, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000008, 'c39cc16f-f7cc-43c4-8a54-5cfd547bb995', 100001, 0,
  'agent.expert.accounts-payable-agent', 24004,
  NULL, 0, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000009, '9946a615-cc1b-425a-8386-a40f764cdb36', 100001, 0,
  'agent.expert.agentic-identity-trust', 7676,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000010, '599f2791-4fad-4c07-8225-4ce3d2a41d94', 100001, 0,
  'agent.expert.agents-orchestrator', 24649,
  NULL, 2, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000011, 'dcf33b9a-0970-4e3a-8e4f-143b9e78b9b1', 100001, 0,
  'agent.expert.authenticity-appraiser', 28123,
  NULL, 3, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000012, '6cb3c5dd-af7b-4497-8f8b-5179a871c0d1', 100001, 0,
  'agent.expert.automation-governance-architect', 6766,
  NULL, 4, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000013, '4a97b367-576c-4633-8cdb-7fb98bceb44f', 100001, 0,
  'agent.expert.blender-addon-engineer', 20474,
  NULL, 0, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000014, '5484b8a7-b7be-47be-8347-736bd37e11ae', 100001, 0,
  'agent.expert.business-strategist', 10188,
  'leader', 5, TIMESTAMPTZ '2026-11-18T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000015, '19a137dd-c953-467c-8ecb-a4c042dbf46e', 100001, 0,
  'agent.expert.change-management-consultant', 41699,
  NULL, 6, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000016, 'fe328029-b747-428f-807f-5325f66c8a84', 100001, 0,
  'agent.expert.chief-executive-officer', 14280,
  'official', 0, TIMESTAMPTZ '2026-11-18T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000017, '4f56ce64-07d4-475e-8c67-3643d1a8ee4d', 100001, 0,
  'agent.expert.chief-financial-officer', 22552,
  NULL, 7, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000018, '38d8a734-d2e7-44ef-8b76-680b29345f4a', 100001, 0,
  'agent.expert.chief-marketing-officer', 28962,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000019, '98c2fc35-3b6d-4bf3-8264-a6d6b5aa3491', 100001, 0,
  'agent.expert.chief-of-staff', 15470,
  NULL, 2, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000020, 'c2abf7cf-2d6d-449a-8c73-fbfd0fd69a05', 100001, 0,
  'agent.expert.chief-operating-officer', 7858,
  NULL, 3, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000021, 'bf91c819-9628-459d-89fc-647e201d6d1a', 100001, 0,
  'agent.expert.chief-product-officer', 1942,
  NULL, 4, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000022, '28f652c7-4c92-47c8-8de0-4ccc64751f63', 100001, 0,
  'agent.expert.chief-technology-officer', 4182,
  NULL, 5, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000023, '6166b7f3-ce08-4085-8c70-f4f44327053e', 100001, 0,
  'agent.expert.corporate-training-designer', 38081,
  NULL, 8, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000024, 'f2e3dbfb-fa58-49a1-86c1-d5e4e7c4a18b', 100001, 0,
  'agent.expert.customer-service', 12843,
  NULL, 9, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000025, '06a00e6d-43d9-4ca1-8a8a-a3d065126095', 100001, 0,
  'agent.expert.customer-success-manager', 7611,
  NULL, 10, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000026, 'a1950044-a7e8-4afd-8d06-28f78ee3b4e4', 100001, 0,
  'agent.expert.data-consolidation-agent', 20023,
  NULL, 11, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000027, '46429701-d707-4ee2-857b-0dd75f0a1446', 100001, 0,
  'agent.expert.data-privacy-officer', 39279,
  NULL, 12, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000028, '92b39bce-a77a-4c0c-8b6c-ce73a0dfe8bf', 100001, 0,
  'agent.expert.design-brand-guardian', 40687,
  NULL, 0, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000029, '8b504032-d9c3-4417-8e1e-1bf8667ea683', 100001, 0,
  'agent.expert.design-image-prompt-engineer', 5042,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000030, '81d43cae-5d64-4192-88ab-4cd468168415', 100001, 0,
  'agent.expert.design-inclusive-visuals-specialist', 7680,
  NULL, 2, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000031, 'b172abc2-3386-4000-8ef9-0c21e0b309c6', 100001, 0,
  'agent.expert.design-persona-walkthrough', 31375,
  NULL, 3, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000032, 'cae93cb7-9809-4fc9-8d19-00d172277d0c', 100001, 0,
  'agent.expert.design-ui-designer', 40062,
  NULL, 4, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000033, 'fa9a4114-c6e2-4432-886c-fb2fb8411cfe', 100001, 0,
  'agent.expert.design-ui-finish-gate-reviewer', 1342,
  NULL, 5, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000034, 'c6e970a7-bf22-46fd-8571-d68e431e4cd4', 100001, 0,
  'agent.expert.design-ux-architect', 21958,
  NULL, 6, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000035, '7aed0396-e785-4159-8ce7-230b68c760a8', 100001, 0,
  'agent.expert.design-ux-researcher', 27878,
  NULL, 7, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000036, 'd996af2b-4787-41c6-894b-e3ef4d400f9a', 100001, 0,
  'agent.expert.design-video-prompt-engineer', 1772,
  NULL, 8, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000037, '51265d2b-48b8-4225-8db2-58773931b135', 100001, 0,
  'agent.expert.design-visual-storyteller', 39230,
  NULL, 9, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000038, 'c5650f7f-bb9a-4c98-8ee6-8f4be3ee3c1c', 100001, 0,
  'agent.expert.design-whimsy-injector', 30933,
  NULL, 10, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000039, 'ca43ceb5-addc-4564-813a-4b92e6c445ed', 100001, 0,
  'agent.expert.economy-designer', 5300,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000040, 'b9e8f5c3-2f4f-43a4-8775-5277bff47eec', 100001, 0,
  'agent.expert.engineering-ai-data-remediation-engineer', 28585,
  NULL, 0, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000041, 'f03bc17b-9ccc-41fe-8fda-a5d54339d118', 100001, 0,
  'agent.expert.engineering-ai-engineer', 40007,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000042, '8ef6f73d-3976-4154-8bec-55e7c3e14f0f', 100001, 0,
  'agent.expert.engineering-api-platform-engineer', 24361,
  NULL, 2, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000043, 'e75af90d-7271-4414-8284-090f024010aa', 100001, 0,
  'agent.expert.engineering-ats-validator-architect', 13184,
  NULL, 3, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000044, '2c7c465d-d91b-4850-89b3-3288e3a156e6', 100001, 0,
  'agent.expert.engineering-autonomous-optimization-architect', 11297,
  NULL, 4, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000045, 'f5623c08-b99c-49ac-8968-07d20e950e21', 100001, 0,
  'agent.expert.engineering-backend-architect', 19141,
  NULL, 5, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000046, '1679f2c8-56a4-4ea2-893b-7d67ac703631', 100001, 0,
  'agent.expert.engineering-china-network-engineer', 25997,
  NULL, 6, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000047, '917ff8c5-16ab-456a-855a-8ec02fd9bf4b', 100001, 0,
  'agent.expert.engineering-cms-developer', 10435,
  NULL, 7, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000048, 'f9c3382c-338b-443f-81d8-f32f76f65aae', 100001, 0,
  'agent.expert.engineering-code-reviewer', 6401,
  NULL, 8, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000049, '6e3873f4-534e-41ab-85e3-99bbc13ccbdc', 100001, 0,
  'agent.expert.engineering-codebase-onboarding-engineer', 11787,
  NULL, 9, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000050, 'c7c82cb0-0082-464a-82a1-36c6ccfd2f72', 100001, 0,
  'agent.expert.engineering-data-engineer', 6781,
  NULL, 10, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000051, '594be92d-0402-4351-8806-ba3363fe2b99', 100001, 0,
  'agent.expert.engineering-data-visualization-engineer', 41168,
  NULL, 11, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000052, 'f34f4416-58d1-4b4d-844d-ac5424565243', 100001, 0,
  'agent.expert.engineering-database-optimizer', 9814,
  NULL, 12, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000053, '17de05ac-5750-4420-83ff-eb55ff30646c', 100001, 0,
  'agent.expert.engineering-database-reliability-engineer', 25993,
  NULL, 13, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000054, '51248458-d7aa-4a22-8d00-4945b071f31e', 100001, 0,
  'agent.expert.engineering-desktop-app-engineer', 14008,
  NULL, 14, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000055, '95fe84d3-3aab-4d36-8dc0-38cb91375413', 100001, 0,
  'agent.expert.engineering-developer-tooling-engineer', 40095,
  NULL, 15, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000056, '3bc2334c-56ee-49b6-84cb-09f4187b3675', 100001, 0,
  'agent.expert.engineering-devops-automator', 34606,
  NULL, 16, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000057, 'b2b30f85-ce1a-425d-8606-f44eeb7eebd4', 100001, 0,
  'agent.expert.engineering-dingtalk-integration-developer', 24415,
  NULL, 17, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000058, '6e041762-ebc3-469d-8efe-fa783325fff8', 100001, 0,
  'agent.expert.engineering-drupal-performance', 37356,
  NULL, 18, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000059, '02bd4c09-e546-4fae-8adb-cc8101dbecee', 100001, 0,
  'agent.expert.engineering-drupal-shopping-cart', 19022,
  NULL, 19, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000060, '48937b27-c9f6-4101-84ae-dc8232250726', 100001, 0,
  'agent.expert.engineering-email-intelligence-engineer', 11456,
  NULL, 20, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000061, '77bb6abe-2afd-41a0-869d-e500bc351c2e', 100001, 0,
  'agent.expert.engineering-embedded-firmware-engineer', 34363,
  NULL, 21, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000062, 'eda80ef2-bf37-481f-8771-91b353f0021b', 100001, 0,
  'agent.expert.engineering-embedded-linux-driver-engineer', 22006,
  NULL, 22, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000063, '120dc6ea-5c40-45c7-8a28-e3fef0eb7071', 100001, 0,
  'agent.expert.engineering-feishu-integration-developer', 27267,
  NULL, 23, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000064, '2b9599bc-3695-4227-8882-505a6e65e20a', 100001, 0,
  'agent.expert.engineering-filament-optimization-specialist', 4786,
  NULL, 24, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000065, '5dbfa8e0-f4c6-4ead-80bd-e99832b82a61', 100001, 0,
  'agent.expert.engineering-finops-engineer', 9692,
  NULL, 25, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000066, '65b50720-7382-4dc5-81f0-293eb2393acd', 100001, 0,
  'agent.expert.engineering-fpga-digital-design-engineer', 33704,
  NULL, 26, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000067, 'bc08757a-a5ed-4ff4-82cf-9c1aaf793a7e', 100001, 0,
  'agent.expert.engineering-frontend-developer', 12272,
  NULL, 27, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000068, '3946ca6c-956a-43b1-8784-741951f97a88', 100001, 0,
  'agent.expert.engineering-gaussdb-expert', 16459,
  NULL, 28, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000069, '79c0c421-3d1d-463b-8867-454315519617', 100001, 0,
  'agent.expert.engineering-git-workflow-master', 10828,
  NULL, 29, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000070, 'c357d950-404f-4d14-84ea-b9411d3ea2bd', 100001, 0,
  'agent.expert.engineering-i18n-engineer', 19064,
  NULL, 30, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000071, '7eb33ea9-4e38-4f8c-8a80-eafcbe55d64d', 100001, 0,
  'agent.expert.engineering-identity-access-engineer', 19457,
  NULL, 31, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000072, '2395dd46-733e-472f-83c8-a66f91db5a1f', 100001, 0,
  'agent.expert.engineering-incident-response-commander', 22180,
  NULL, 32, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000073, '12c1c62c-d441-4ac8-8588-1d17c0d35ce7', 100001, 0,
  'agent.expert.engineering-iot-fleet-engineer', 35011,
  NULL, 33, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000074, '59e8f599-a574-4759-8abb-322133d11dc9', 100001, 0,
  'agent.expert.engineering-iot-solution-architect', 9415,
  NULL, 34, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000075, '14ceae3b-df9b-45f6-8e1e-eddfb2dc50ce', 100001, 0,
  'agent.expert.engineering-it-service-manager', 16451,
  NULL, 35, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000076, '2b1458a8-486c-4ab8-81fc-176e14c282b8', 100001, 0,
  'agent.expert.engineering-knowledge-graph-engineer', 13909,
  NULL, 36, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000077, '5b208a53-bfb9-4f25-8235-00d83c7026ba', 100001, 0,
  'agent.expert.engineering-llm-post-training-engineer', 40895,
  NULL, 37, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000078, '76474755-dcb6-4907-81de-56ed46a9557c', 100001, 0,
  'agent.expert.engineering-mechanical-design-engineer', 37306,
  NULL, 38, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000079, '0272efd0-382a-4e2f-8dc3-664b43d617e8', 100001, 0,
  'agent.expert.engineering-minimal-change-engineer', 24927,
  NULL, 39, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000080, '9654094a-ae20-4b3d-85f9-4f2315e6e7ba', 100001, 0,
  'agent.expert.engineering-mobile-app-builder', 30186,
  NULL, 40, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000081, '7d1887a3-e6a8-4de4-85fa-d290dbecfcf1', 100001, 0,
  'agent.expert.engineering-mobile-release-engineer', 40844,
  NULL, 41, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000082, 'd2e5731e-b069-4335-8d9c-5015f8aa712d', 100001, 0,
  'agent.expert.engineering-multi-agent-systems-architect', 36107,
  NULL, 42, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000083, '4a143790-31bb-4ab5-8671-8aa2b1490d7e', 100001, 0,
  'agent.expert.engineering-network-engineer', 9497,
  NULL, 43, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000084, 'b5146097-b020-4f73-80f4-6c2b3404f050', 100001, 0,
  'agent.expert.engineering-network-engineer-china', 20204,
  NULL, 44, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000085, '3179fd43-3aa1-4cd9-872a-ed17e8d1cd72', 100001, 0,
  'agent.expert.engineering-orgscript-engineer', 5333,
  NULL, 45, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000086, '711117d8-3ade-45c1-8703-d2bc97ea7eaf', 100001, 0,
  'agent.expert.engineering-payments-billing-engineer', 4949,
  NULL, 46, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000087, 'ec6dbc86-0fce-46a9-80b5-136374e5ced9', 100001, 0,
  'agent.expert.engineering-pc-host-engineer', 34508,
  NULL, 47, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000088, '9daa804a-6ba0-475f-8771-4352bb06e13e', 100001, 0,
  'agent.expert.engineering-pdf-engine-architect', 10595,
  NULL, 48, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000089, '8727ca32-5ed0-42f5-84f3-004a35f6c63e', 100001, 0,
  'agent.expert.engineering-platform-engineer', 14557,
  NULL, 49, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000090, 'bd5cc3e9-7bfc-4b18-85fb-b3598bbb8a01', 100001, 0,
  'agent.expert.engineering-privacy-engineer', 40891,
  NULL, 50, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000091, 'bb86de0c-fb3a-4f7c-8483-b3c7a6626f3f', 100001, 0,
  'agent.expert.engineering-prompt-engineer', 38853,
  NULL, 51, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000092, '1134a8f3-4408-455b-8f9d-01dcc8630263', 100001, 0,
  'agent.expert.engineering-rag-pipeline-engineer', 21624,
  NULL, 52, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000093, 'e5e1031a-dd4f-4044-86f0-b928e299c4b9', 100001, 0,
  'agent.expert.engineering-rapid-prototyper', 35408,
  NULL, 53, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000094, '401af42a-4006-4540-858a-f58090c4bfbf', 100001, 0,
  'agent.expert.engineering-realtime-collaboration-engineer', 10469,
  NULL, 54, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000095, '7aa914b7-129b-4023-8c24-23749c76a74e', 100001, 0,
  'agent.expert.engineering-rust-refactoring-specialist', 25241,
  NULL, 55, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000096, 'b9272ede-d9b8-4b1a-8188-e42cb20dc743', 100001, 0,
  'agent.expert.engineering-search-relevance-engineer', 6992,
  NULL, 56, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000097, 'b689c83e-fdcc-42f9-8446-1f8a5f3d82b1', 100001, 0,
  'agent.expert.engineering-section-508-specialist', 37616,
  NULL, 57, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000098, 'd9eff5d4-0aed-4823-85fc-52ef2604da6c', 100001, 0,
  'agent.expert.engineering-security-engineer', 22257,
  NULL, 58, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000099, 'b19866f9-06b1-407e-84e1-dc8963f95ff0', 100001, 0,
  'agent.expert.engineering-senior-developer', 38056,
  NULL, 59, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000100, '841669c9-5e4f-44b0-87aa-0abc91c56d76', 100001, 0,
  'agent.expert.engineering-servicenow-developer-mentor', 3418,
  NULL, 60, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000101, 'ab7e1597-0c7d-4774-8011-c68294cebbc9', 100001, 0,
  'agent.expert.engineering-software-architect', 23727,
  NULL, 61, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000102, '5d828d2e-02e2-4010-88a0-345c4469bd84', 100001, 0,
  'agent.expert.engineering-solidity-smart-contract-engineer', 37224,
  NULL, 62, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000103, 'e96ab1a0-952d-43bb-85ce-c58b0e4524a3', 100001, 0,
  'agent.expert.engineering-sre', 15241,
  NULL, 63, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000104, 'e168e853-acc9-4a07-81b4-c737c0bd1fef', 100001, 0,
  'agent.expert.engineering-technical-writer', 5714,
  NULL, 64, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000105, '42c41572-376e-4d8f-80f6-5465754f30e3', 100001, 0,
  'agent.expert.engineering-threat-detection-engineer', 2180,
  NULL, 65, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000106, 'cc74c4d9-3eef-4970-8cc6-6d6586ef45c0', 100001, 0,
  'agent.expert.engineering-universal-document-compiler', 9663,
  NULL, 66, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000107, 'fcf6e624-510f-4a05-8b76-2a50f979c0d2', 100001, 0,
  'agent.expert.engineering-uswds-developer', 34339,
  NULL, 67, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000108, '29539d74-ce25-4203-84b2-1e6c5188d166', 100001, 0,
  'agent.expert.engineering-video-streaming-engineer', 16102,
  NULL, 68, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000109, '9d406247-2272-403b-88ae-f384bd93b46f', 100001, 0,
  'agent.expert.engineering-voice-ai-integration-engineer', 18793,
  NULL, 69, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000110, '1f5e9422-4385-4eb1-8f9b-eeb6bedc2fb8', 100001, 0,
  'agent.expert.engineering-webassembly-engineer', 11538,
  NULL, 70, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000111, '6e1fc727-a365-41eb-8240-97ac04a95436', 100001, 0,
  'agent.expert.engineering-wechat-mini-program-developer', 18941,
  NULL, 71, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000112, '040ddec6-c42e-4287-8251-bd1637ba538b', 100001, 0,
  'agent.expert.engineering-wordpress-performance', 28476,
  NULL, 72, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000113, 'ff1a4d50-1f39-44e8-8f98-2d2983620dd5', 100001, 0,
  'agent.expert.engineering-wordpress-shopping-cart', 22609,
  NULL, 73, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000114, 'e61b1f65-a38d-4728-8f98-ef210b63f329', 100001, 0,
  'agent.expert.esg-sustainability-officer', 6099,
  NULL, 13, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000115, 'a32915ef-ce57-4772-8191-064ee3722f07', 100001, 0,
  'agent.expert.finance-bookkeeper-controller', 18319,
  NULL, 0, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000116, '2ff86afd-eceb-46df-8c5e-223aa254700e', 100001, 0,
  'agent.expert.finance-financial-analyst', 10750,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000117, 'b4284926-0003-47e2-8e59-e44a941ef8b0', 100001, 0,
  'agent.expert.finance-financial-forecaster', 4078,
  NULL, 2, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000118, 'b8b2fb25-7b6b-4b99-8b0f-73b809b0cce8', 100001, 0,
  'agent.expert.finance-fpa-analyst', 15755,
  NULL, 3, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000119, '3ce13ddb-d27f-47fc-8ca4-1b20e25cd365', 100001, 0,
  'agent.expert.finance-fraud-detector', 5060,
  NULL, 4, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000120, 'f4df3b54-fc07-4306-8e34-452558788637', 100001, 0,
  'agent.expert.finance-hk-stock-compliance-reviewer', 16743,
  NULL, 5, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000121, '5000df6a-93ee-4cee-8ed2-ad3d2e9db737', 100001, 0,
  'agent.expert.finance-investment-researcher', 18614,
  'leader', 6, TIMESTAMPTZ '2026-11-18T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000122, 'b8ea3394-48ae-44cb-8998-be104c97ebe2', 100001, 0,
  'agent.expert.finance-invoice-manager', 32215,
  NULL, 7, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000123, 'a8418e4d-6915-4a0c-85bb-c79ad9c9f498', 100001, 0,
  'agent.expert.finance-tax-strategist', 18130,
  NULL, 8, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000124, '3316c79f-e725-4e21-8506-415cc65c44ca', 100001, 0,
  'agent.expert.game-audio-engineer', 17842,
  NULL, 2, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000125, 'bac6c022-7af1-4f88-859b-7e45923c04d8', 100001, 0,
  'agent.expert.game-designer', 32093,
  NULL, 3, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000126, '6d7b38cb-330b-419c-8707-fc292dd6520e', 100001, 0,
  'agent.expert.gaokao-college-advisor', 21552,
  NULL, 14, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000127, '3755a7c3-b8bc-4795-8614-d2e258c84866', 100001, 0,
  'agent.expert.gis-3d-scene-developer', 14267,
  NULL, 0, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000128, '0eba827d-bda4-444d-8839-a92497764d1f', 100001, 0,
  'agent.expert.gis-analyst', 36610,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000129, '590bf1f1-68d3-4e05-8ab5-df7ad9927bb4', 100001, 0,
  'agent.expert.gis-bim-specialist', 22219,
  NULL, 2, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000130, 'da29f9f9-d7d7-4150-8baf-ee89232b31e9', 100001, 0,
  'agent.expert.gis-cartography-designer', 18338,
  NULL, 3, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000131, 'cb8b65bb-1961-43d5-8622-d0a8dc208355', 100001, 0,
  'agent.expert.gis-drone-reality-mapping', 41539,
  NULL, 4, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000132, '27c50d4e-1709-47ba-86bb-8db9d778735d', 100001, 0,
  'agent.expert.gis-geoai-ml-engineer', 34113,
  NULL, 5, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000133, '055953f2-2c0e-4e98-8321-afc4c5c34624', 100001, 0,
  'agent.expert.gis-geoprocessing-specialist', 38748,
  NULL, 6, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000134, '405cc131-fae7-4d04-8131-4f27b451d7d1', 100001, 0,
  'agent.expert.gis-qa-engineer', 1078,
  NULL, 7, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000135, 'c8117116-f850-4dab-85e5-2fe25847b465', 100001, 0,
  'agent.expert.gis-solution-engineer', 33278,
  NULL, 8, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000136, '0a16c95d-a897-4d50-8aab-7443315ddeb1', 100001, 0,
  'agent.expert.gis-spatial-data-engineer', 9822,
  NULL, 9, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000137, '590de079-5409-49b4-8df0-de486aec6acb', 100001, 0,
  'agent.expert.gis-spatial-data-scientist', 37922,
  NULL, 10, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000138, 'f6ee8524-24da-4b5b-8431-2fc2e80a51d9', 100001, 0,
  'agent.expert.gis-technical-consultant', 39762,
  NULL, 11, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000139, 'ab801dfa-1966-46bc-8874-7e2ec4945f5d', 100001, 0,
  'agent.expert.gis-web-gis-developer', 39997,
  NULL, 12, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000140, '3fb090e6-a784-489f-80ed-21c5cf7dddd0', 100001, 0,
  'agent.expert.godot-gameplay-scripter', 23103,
  NULL, 4, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000141, '2262355c-c11d-4928-8db6-ad8ef66a45e0', 100001, 0,
  'agent.expert.godot-multiplayer-engineer', 23158,
  NULL, 5, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000142, '64dc7982-280a-46b2-8f75-f68986c70e06', 100001, 0,
  'agent.expert.godot-shader-developer', 20673,
  NULL, 6, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000143, 'bf1feeea-81f3-4540-80e9-1995e499501a', 100001, 0,
  'agent.expert.government-digital-presales-consultant', 15890,
  NULL, 15, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000144, '4b9d971f-c6a5-4ae1-83e2-6a6e7f8be49c', 100001, 0,
  'agent.expert.grant-writer', 14467,
  NULL, 16, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000145, 'e31d14b7-6fba-4bd8-817f-fc6293cfafef', 100001, 0,
  'agent.expert.healthcare-aging-parent-care-companion', 7922,
  NULL, 17, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000146, 'f2ef73f4-44ea-4369-8b63-90f28c2472eb', 100001, 0,
  'agent.expert.healthcare-clinical-evidence-agent', 34238,
  NULL, 0, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000147, '807d47e5-652b-4ad4-82d7-4641c1ffe98d', 100001, 0,
  'agent.expert.healthcare-customer-service', 13834,
  NULL, 18, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000148, 'ee990737-5639-4466-8340-7ac757cb311d', 100001, 0,
  'agent.expert.healthcare-innovation-strategist', 26036,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000149, '04cf1d23-2fe0-4c31-8804-85ec819a6e84', 100001, 0,
  'agent.expert.healthcare-marketing-compliance', 23951,
  NULL, 19, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000150, '6b0b7f07-6436-4244-87a4-670d428919b1', 100001, 0,
  'agent.expert.healthcare-sovereign-health-systems-agent', 28172,
  NULL, 2, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000151, 'bae3c599-c7b2-4379-844b-83cb1b17baca', 100001, 0,
  'agent.expert.hospitality-guest-services', 10893,
  NULL, 20, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000152, 'cc4ae7f7-c573-478c-8f08-ea6d9d6fe2d9', 100001, 0,
  'agent.expert.hr-onboarding', 37160,
  NULL, 21, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000153, '5e0c83c4-3764-45d8-86f9-8e6949b28514', 100001, 0,
  'agent.expert.hr-performance-reviewer', 4429,
  NULL, 0, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000154, '9835f7d8-77e9-413a-80e8-fcbf4ad9976e', 100001, 0,
  'agent.expert.hr-recruiter', 37339,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000155, 'b51fd55d-06f8-43e5-8662-fdf7e31b02ca', 100001, 0,
  'agent.expert.identity-graph-operator', 15959,
  NULL, 22, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000156, 'd1cbaee2-9d36-4c1b-8144-f6aec6936c31', 100001, 0,
  'agent.expert.language-translator', 3368,
  NULL, 23, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000157, '8da7e68b-7b20-4ead-8243-77b4d1cb0812', 100001, 0,
  'agent.expert.legal-billing-time-tracking', 38770,
  NULL, 24, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000158, 'bdb9ea1d-86f2-4d59-861b-f389146d319d', 100001, 0,
  'agent.expert.legal-client-intake', 3892,
  NULL, 25, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000159, '7052ff59-604e-4da1-8f67-1d4922c6706a', 100001, 0,
  'agent.expert.legal-contract-reviewer', 29569,
  'leader', 0, TIMESTAMPTZ '2026-11-18T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000160, '0f13c63b-133f-4838-86d2-de4abec65526', 100001, 0,
  'agent.expert.legal-document-review', 27653,
  NULL, 26, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000161, '70f766ad-0520-4516-88f2-3f05f8298232', 100001, 0,
  'agent.expert.legal-policy-writer', 25297,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000162, '12b9d97c-31a1-4faf-8342-ad7f37e2eea9', 100001, 0,
  'agent.expert.level-designer', 32151,
  NULL, 7, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000163, '8c9c7919-8ade-46a5-8900-5c375f0a560b', 100001, 0,
  'agent.expert.livestock-archive-auditor', 19181,
  NULL, 27, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000164, '91fda860-9a99-466f-835d-ab180ba98102', 100001, 0,
  'agent.expert.loan-officer-assistant', 18257,
  NULL, 28, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000165, '2bb3363e-709d-404d-8dbf-7c54fd5b34aa', 100001, 0,
  'agent.expert.lsp-index-engineer', 2959,
  NULL, 29, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000166, '13bb53c4-607e-4b6a-884d-f50d80448ed5', 100001, 0,
  'agent.expert.ma-integration-manager', 19542,
  NULL, 30, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000167, 'e66d1378-d0c8-4335-8634-23788a9fe89d', 100001, 0,
  'agent.expert.macos-spatial-metal-engineer', 6131,
  NULL, 0, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000168, '7d42add0-b80f-4d0c-8f3d-f5f5d0522730', 100001, 0,
  'agent.expert.marketing-aeo-foundations', 41485,
  NULL, 0, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000169, '551a2f58-a21a-4165-8808-3988207a2fe6', 100001, 0,
  'agent.expert.marketing-agentic-search-optimizer', 17225,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000170, 'cdc946dd-3b36-46a8-8c5b-d40d5a0002bf', 100001, 0,
  'agent.expert.marketing-ai-citation-strategist', 21952,
  NULL, 2, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000171, 'a8c5eb07-f3dc-46bc-86bb-cd9e6de27f2e', 100001, 0,
  'agent.expert.marketing-app-store-optimizer', 10891,
  NULL, 3, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000172, '48484137-f4b9-4b0f-8cb6-a5128dafd810', 100001, 0,
  'agent.expert.marketing-baidu-seo-specialist', 5801,
  NULL, 4, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000173, 'cf1d3b12-b653-4129-8535-4eae0c239e96', 100001, 0,
  'agent.expert.marketing-bilibili-content-strategist', 5533,
  NULL, 5, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000174, '02ef4f45-2141-44e1-8758-3515bdb75694', 100001, 0,
  'agent.expert.marketing-bilibili-strategist', 13865,
  NULL, 6, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000175, '7608e7a7-9cc7-4a64-85e7-ae2d1f36df77', 100001, 0,
  'agent.expert.marketing-book-co-author', 23364,
  NULL, 7, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000176, '65c5a86f-2a09-4a52-8960-9627916b1440', 100001, 0,
  'agent.expert.marketing-carousel-growth-engine', 24702,
  NULL, 8, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000177, '74511025-b3e5-41b4-85b4-b0b3dccf5cc6', 100001, 0,
  'agent.expert.marketing-china-ecommerce-operator', 33803,
  NULL, 9, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000178, 'b8075def-b138-435d-8b81-dcaa8bb9f4a7', 100001, 0,
  'agent.expert.marketing-china-market-localization-strategist', 23555,
  NULL, 10, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000179, '096e6e55-88db-49c6-8a7d-d0ea414bc2fc', 100001, 0,
  'agent.expert.marketing-content-creator', 29184,
  'leader', 11, TIMESTAMPTZ '2026-11-18T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000180, '2f3f5e8a-d7d9-4958-8587-22aad0e5fcfc', 100001, 0,
  'agent.expert.marketing-cross-border-ecommerce', 25876,
  NULL, 12, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000181, '182f61c8-8577-4c2e-878a-4774e31b8099', 100001, 0,
  'agent.expert.marketing-daily-news-briefing', 37987,
  NULL, 13, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000182, '22375671-5909-4a47-86df-1343d69b9083', 100001, 0,
  'agent.expert.marketing-developer-community-builder', 41392,
  NULL, 14, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000183, '298ae265-5962-4c90-8b9b-6e94238b116c', 100001, 0,
  'agent.expert.marketing-douyin-strategist', 20325,
  NULL, 15, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000184, 'cf10cb02-3ee2-403f-87c4-46fc7b928448', 100001, 0,
  'agent.expert.marketing-ecommerce-operator', 11931,
  'leader', 16, TIMESTAMPTZ '2026-11-18T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000185, 'f835cd01-56e0-4c58-8708-2091bd92996d', 100001, 0,
  'agent.expert.marketing-email-strategist', 9075,
  NULL, 17, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000186, '25c75b2f-864b-4a27-8995-2149a411fb29', 100001, 0,
  'agent.expert.marketing-global-podcast-strategist', 34082,
  NULL, 18, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000187, '6b98abdb-6671-4261-830a-ead7232d4e71', 100001, 0,
  'agent.expert.marketing-growth-hacker', 21289,
  NULL, 19, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000188, '3d5a75f6-c15e-4fbf-8f9f-ea0d10e11c7f', 100001, 0,
  'agent.expert.marketing-instagram-curator', 23021,
  NULL, 20, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000189, '16102401-394d-4a76-8d00-a875cc4c975f', 100001, 0,
  'agent.expert.marketing-knowledge-commerce-strategist', 12150,
  NULL, 21, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000190, '73c60605-2a6a-4ced-8e64-469b3c7285de', 100001, 0,
  'agent.expert.marketing-kuaishou-strategist', 13460,
  NULL, 22, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000191, '36ea3d75-4090-4973-80f9-686449e5d64a', 100001, 0,
  'agent.expert.marketing-linkedin-content-creator', 36074,
  NULL, 23, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000192, '78cd9c18-ad8e-40e1-848b-38bb54f08d67', 100001, 0,
  'agent.expert.marketing-livestream-commerce-coach', 30673,
  NULL, 24, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000193, 'da9d20a6-d69a-4880-8902-1a5c3898d606', 100001, 0,
  'agent.expert.marketing-multi-platform-publisher', 13082,
  NULL, 25, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000194, '945e8613-fa88-4502-8a6a-b550816e1721', 100001, 0,
  'agent.expert.marketing-podcast-strategist', 37312,
  NULL, 26, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000195, '5d1336f9-f576-4069-895d-de629783bc8a', 100001, 0,
  'agent.expert.marketing-pr-communications-manager', 17972,
  NULL, 27, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000196, 'e6f5246f-b033-4c11-8543-99ee15eface7', 100001, 0,
  'agent.expert.marketing-private-domain-operator', 16004,
  NULL, 28, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000197, '811a7936-f29f-477e-8b09-e20d8d91aaa7', 100001, 0,
  'agent.expert.marketing-reddit-community-builder', 33899,
  NULL, 29, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000198, 'd39968f8-d40c-4920-8d3e-b82c099f5ecc', 100001, 0,
  'agent.expert.marketing-search-growth-orchestrator', 11329,
  NULL, 30, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000199, '679e7dbc-3aba-4661-8763-b91ee29f9116', 100001, 0,
  'agent.expert.marketing-seo-specialist', 11605,
  NULL, 31, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000200, '97f53a01-81f2-45b8-89fd-e80faa2eace5', 100001, 0,
  'agent.expert.marketing-short-video-editing-coach', 16507,
  NULL, 32, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000201, '5b8927e7-e259-4419-8ef2-51202e5fb392', 100001, 0,
  'agent.expert.marketing-social-media-strategist', 17841,
  NULL, 33, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000202, 'ce0c5338-d0b1-4d88-890e-22c2513197a5', 100001, 0,
  'agent.expert.marketing-tiktok-strategist', 21603,
  NULL, 34, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000203, '122427d7-3674-4591-8bfd-9b6a4ccec5d9', 100001, 0,
  'agent.expert.marketing-twitter-engager', 5508,
  NULL, 35, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000204, '6cf07d4e-dcda-4385-8b8a-10ad3df63d3e', 100001, 0,
  'agent.expert.marketing-video-optimization-specialist', 13616,
  NULL, 36, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000205, 'e8413e01-ad4f-4f4a-8ffd-fc2736700f30', 100001, 0,
  'agent.expert.marketing-wechat-official-account', 35348,
  NULL, 37, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000206, '481d3705-7a24-45ac-8b64-54055499e4a2', 100001, 0,
  'agent.expert.marketing-wechat-operator', 1132,
  NULL, 38, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000207, 'ee6d3ac6-375f-4e75-8b1a-7215c1e5eb51', 100001, 0,
  'agent.expert.marketing-weibo-strategist', 26026,
  NULL, 39, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000208, '634bec6f-7b6d-41b9-8795-d2c677433fac', 100001, 0,
  'agent.expert.marketing-weixin-channels-strategist', 10304,
  NULL, 40, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000209, '7d88c73c-3653-4136-81a7-21ef971835f3', 100001, 0,
  'agent.expert.marketing-x-twitter-intelligence-analyst', 5965,
  NULL, 41, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000210, '1e226efb-0598-42c3-82c6-84e671c63096', 100001, 0,
  'agent.expert.marketing-xiaohongshu-operator', 13487,
  NULL, 42, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000211, 'c200eb1a-3fc4-475f-8684-4499347be48e', 100001, 0,
  'agent.expert.marketing-xiaohongshu-specialist', 7854,
  NULL, 43, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000212, 'b9d48a07-0125-48b9-8dc8-8e495fce8a1f', 100001, 0,
  'agent.expert.marketing-zhihu-strategist', 9813,
  NULL, 44, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000213, '9fc0e085-7e4d-45de-863a-76e61ff213f5', 100001, 0,
  'agent.expert.medical-billing-coding-specialist', 29417,
  NULL, 31, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000214, '50a1664d-925e-4bb5-8ee1-c87878ca6714', 100001, 0,
  'agent.expert.narrative-designer', 41757,
  NULL, 8, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000215, 'c42de9cc-b1f8-4b01-8a99-242fb7b4a4db', 100001, 0,
  'agent.expert.operations-manager', 7838,
  NULL, 32, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000216, '4676ca31-3c27-4ece-83f7-f65fb86d27e5', 100001, 0,
  'agent.expert.organizational-psychologist', 32887,
  NULL, 33, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000217, '96e1f84f-46b8-4e70-8748-e2de40270228', 100001, 0,
  'agent.expert.paid-media-auditor', 39327,
  NULL, 0, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000218, '864c70fa-67c7-43a9-8440-8f27edb1ffe4', 100001, 0,
  'agent.expert.paid-media-creative-strategist', 17868,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000219, 'fef6ef97-e9ea-4488-8dd6-76c8604b1b5f', 100001, 0,
  'agent.expert.paid-media-paid-social-strategist', 4568,
  NULL, 2, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000220, '25d6bdee-620b-44f7-8c9d-d5275209f40e', 100001, 0,
  'agent.expert.paid-media-ppc-strategist', 17813,
  NULL, 3, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000221, '55bd5bed-f68f-418c-8073-4e4a6d32b609', 100001, 0,
  'agent.expert.paid-media-programmatic-buyer', 13981,
  NULL, 4, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000222, '1032c468-a9a5-47d9-8793-3306e0a51944', 100001, 0,
  'agent.expert.paid-media-search-query-analyst', 12141,
  NULL, 5, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000223, '8bbfd6ca-8efe-45b3-8efc-6298e011918d', 100001, 0,
  'agent.expert.paid-media-tracking-specialist', 1389,
  NULL, 6, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000224, 'e5a0d900-fffd-405e-82f7-0a40c3b6feb2', 100001, 0,
  'agent.expert.personal-growth-mentor', 21209,
  NULL, 34, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000225, '4399ab9d-ca34-4a4d-87cd-abb5c7ca73da', 100001, 0,
  'agent.expert.product-behavioral-nudge-engine', 9367,
  NULL, 0, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000226, '034dfa6b-b7a0-4c0c-888a-3dded5f782d1', 100001, 0,
  'agent.expert.product-dx-engineer', 34419,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000227, 'f75ec543-15c7-4fcb-8836-0bceb1d1dae9', 100001, 0,
  'agent.expert.product-feedback-synthesizer', 6894,
  NULL, 2, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000228, '2bf72ffd-557b-4fa1-8fbf-2818b74066c4', 100001, 0,
  'agent.expert.product-manager', 1897,
  'official', 3, TIMESTAMPTZ '2026-11-18T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000229, 'd9e1d9ce-ead4-4244-8236-07655cc18eef', 100001, 0,
  'agent.expert.product-sprint-prioritizer', 4059,
  NULL, 4, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000230, 'b985dd6b-b4b6-4a5b-85d3-225b519f5a65', 100001, 0,
  'agent.expert.product-trend-researcher', 13768,
  NULL, 5, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000231, 'a1aa0b9a-5e5e-4fc2-839d-6a5f626261d7', 100001, 0,
  'agent.expert.project-management-experiment-tracker', 23928,
  NULL, 0, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000232, 'b31d9660-71f2-43de-8804-686dc3cfba66', 100001, 0,
  'agent.expert.project-management-jira-workflow-steward', 15640,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000233, 'cfd1e993-ba99-46d3-812f-f528ab9d5b3b', 100001, 0,
  'agent.expert.project-management-meeting-notes-specialist', 40664,
  NULL, 2, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000234, 'bb08eb92-c236-4c76-8be1-72fb242d590e', 100001, 0,
  'agent.expert.project-management-project-shepherd', 14743,
  NULL, 3, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000235, '5e7b7884-90d6-423d-8f94-558c680e44d2', 100001, 0,
  'agent.expert.project-management-studio-operations', 2113,
  NULL, 4, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000236, '76ad9af9-6df3-4dc1-8db7-b97d91c69f23', 100001, 0,
  'agent.expert.project-management-studio-producer', 20029,
  NULL, 5, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000237, '1c6d6869-0aee-45d7-8036-7627b78f9dbc', 100001, 0,
  'agent.expert.project-manager-senior', 10598,
  NULL, 6, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000238, '7bac61a4-620b-4c2c-8098-45bc6aa7b3f8', 100001, 0,
  'agent.expert.prompt-engineer', 1743,
  NULL, 35, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000239, '8a2e42e1-1dd1-4ef4-8794-2fe8feb0805d', 100001, 0,
  'agent.expert.real-estate-buyer-seller', 3144,
  NULL, 36, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000240, '68e22824-ca16-47e7-8ff4-b69bd680d2b5', 100001, 0,
  'agent.expert.recruitment-specialist', 27596,
  NULL, 37, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000241, 'ca778210-07bb-42ce-8c69-b5e3c4d73fdd', 100001, 0,
  'agent.expert.report-distribution-agent', 27084,
  NULL, 38, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000242, 'f5fb17ec-9e14-4eef-8dd3-c67adcf0c0f6', 100001, 0,
  'agent.expert.research-synthesist', 23339,
  NULL, 0, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000243, 'a2346402-29fd-4f93-8811-9cf9901061a5', 100001, 0,
  'agent.expert.resume-tailor', 30089,
  NULL, 39, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000244, 'b2912e8c-934f-4253-819b-e7e782dcb43d', 100001, 0,
  'agent.expert.retail-customer-returns', 37840,
  NULL, 40, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000245, '5d3eea0a-5833-4b76-8811-57d1c5bef98e', 100001, 0,
  'agent.expert.roblox-avatar-creator', 20311,
  NULL, 9, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000246, '72f4dd97-f818-4788-8941-b60991200ac1', 100001, 0,
  'agent.expert.roblox-experience-designer', 29592,
  NULL, 10, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000247, 'fb2f310e-1e50-43fd-8e6e-96642440db85', 100001, 0,
  'agent.expert.roblox-systems-scripter', 7466,
  NULL, 11, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000248, 'f86e0e83-2d66-40fb-8d6a-d8e3f04ef082', 100001, 0,
  'agent.expert.sales-account-strategist', 17740,
  NULL, 0, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000249, '90b058aa-145c-4784-840f-7764bc7463e1', 100001, 0,
  'agent.expert.sales-coach', 30421,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000250, '63577087-8a61-4e4a-82d6-bda18ee0342e', 100001, 0,
  'agent.expert.sales-data-extraction-agent', 7689,
  NULL, 41, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000251, '10c905ba-34d0-435c-88a7-17f643db1c3b', 100001, 0,
  'agent.expert.sales-deal-strategist', 22998,
  NULL, 2, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000252, 'c9f186e5-547b-44d6-88f6-07a83f47b108', 100001, 0,
  'agent.expert.sales-discovery-coach', 9313,
  NULL, 3, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000253, '4c139ea7-98b4-4750-8046-2825d1160b86', 100001, 0,
  'agent.expert.sales-engineer', 11568,
  NULL, 4, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000254, 'ee976c5d-af55-4168-82ff-80b68b452eed', 100001, 0,
  'agent.expert.sales-offer-lead-gen-strategist', 16763,
  NULL, 5, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000255, 'a58bde87-7134-4b46-8c44-d6d7b8eabd63', 100001, 0,
  'agent.expert.sales-outbound-strategist', 9223,
  NULL, 6, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000256, '3d6ad361-c17a-4cf8-8b2e-0dbd5a65ad3a', 100001, 0,
  'agent.expert.sales-outreach', 1930,
  NULL, 42, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000257, 'f8b19391-387f-46d2-8d20-ac44e5f109c1', 100001, 0,
  'agent.expert.sales-pipeline-analyst', 39595,
  NULL, 7, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000258, '97af3f6e-8400-4ac9-883b-86f671c11add', 100001, 0,
  'agent.expert.sales-proposal-strategist', 40265,
  NULL, 8, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000259, '55f95fd1-dfa9-4ff9-846f-f8306eb1384b', 100001, 0,
  'agent.expert.security-ai-generated-code-auditor', 26228,
  NULL, 0, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000260, '631695e2-3cbf-417f-8bde-2c738c5eab37', 100001, 0,
  'agent.expert.security-appsec-engineer', 1912,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000261, '306dc7e9-0c34-4ad2-8866-3cb5236b21a3', 100001, 0,
  'agent.expert.security-architect', 16521,
  NULL, 2, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000262, '0ec42db1-23bd-4123-85d0-9cc16bcfa816', 100001, 0,
  'agent.expert.security-blockchain-security-auditor', 36769,
  NULL, 3, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000263, 'f48ca298-4981-44f2-8d6b-f0fac0d2193c', 100001, 0,
  'agent.expert.security-cloud-security-architect', 38491,
  NULL, 4, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000264, 'c2b39513-879d-4f58-829d-f7ee8848df8f', 100001, 0,
  'agent.expert.security-compliance-auditor', 3002,
  NULL, 5, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000265, '456c061e-c0da-4d5f-819d-46e13563aae5', 100001, 0,
  'agent.expert.security-incident-responder', 38576,
  NULL, 6, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000266, '32eb68be-4f81-4d52-8dce-2095e089217e', 100001, 0,
  'agent.expert.security-penetration-tester', 35661,
  NULL, 7, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000267, '37bb5004-cb0f-4cb7-8dc2-d9b3d3fdb3ec', 100001, 0,
  'agent.expert.security-secrets-credential-engineer', 17806,
  NULL, 8, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000268, 'c6deef56-7723-4275-8118-703f8d7f5e28', 100001, 0,
  'agent.expert.security-senior-secops', 19739,
  NULL, 9, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000269, 'caf13363-50ef-4d3e-8555-023f1e8e537e', 100001, 0,
  'agent.expert.security-threat-detection-engineer', 27534,
  NULL, 10, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000270, 'ea40b123-ad8b-4ab8-81ac-f3b190ef5d2c', 100001, 0,
  'agent.expert.security-threat-intelligence-analyst', 34671,
  NULL, 11, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000271, 'a67cbd8a-8006-4c26-8b26-d751d98dd264', 100001, 0,
  'agent.expert.specialized-ai-policy-writer', 41645,
  NULL, 43, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000272, 'c13030be-19c0-4dbd-85b6-a895950d0565', 100001, 0,
  'agent.expert.specialized-chief-of-staff', 6485,
  NULL, 44, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000273, '4a5f52a9-4d99-48fe-8999-668876449d48', 100001, 0,
  'agent.expert.specialized-civil-engineer', 7991,
  NULL, 45, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000274, '57350f60-8749-4f89-8b21-282ba2e12199', 100001, 0,
  'agent.expert.specialized-codebase-archaeologist', 37702,
  NULL, 46, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000275, '46b43209-55b7-4544-8883-c6de4c5ee1e0', 100001, 0,
  'agent.expert.specialized-cultural-intelligence-strategist', 33440,
  NULL, 47, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000276, '3e4ac62a-f43d-44c5-82dd-ba168ecd30d5', 100001, 0,
  'agent.expert.specialized-developer-advocate', 2429,
  NULL, 48, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000277, '811ff0ef-5bbb-4e98-8ee6-8f52d306dd92', 100001, 0,
  'agent.expert.specialized-document-generator', 9187,
  NULL, 49, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000278, 'cc5a1498-1dfc-4c1d-8980-c2e1d0f72c4c', 100001, 0,
  'agent.expert.specialized-fedramp-rmf-compliance', 10821,
  NULL, 50, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000279, 'e6f3e429-f6d0-40b1-8833-0fccd75d87cd', 100001, 0,
  'agent.expert.specialized-focus-music-architect', 17982,
  NULL, 51, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000280, '7604325c-3fbc-4e26-8520-bf857ab728ed', 100001, 0,
  'agent.expert.specialized-french-consulting-market', 33828,
  NULL, 52, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000281, '8c216332-cbfe-4343-8492-160e78b1db42', 100001, 0,
  'agent.expert.specialized-korean-business-navigator', 6378,
  NULL, 53, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000282, '0e956a52-3975-4b2d-862d-5a8b30239e4d', 100001, 0,
  'agent.expert.specialized-master-plan-architect', 14478,
  NULL, 54, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000283, 'ab8cb738-f0de-4712-8d84-c88be637d7bc', 100001, 0,
  'agent.expert.specialized-mcp-builder', 19405,
  NULL, 55, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000284, '96147f72-f953-4969-8cef-e74c6fd1e192', 100001, 0,
  'agent.expert.specialized-meeting-assistant', 18344,
  NULL, 56, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000285, '894a3ff5-68fe-4486-833e-2696e5fa391c', 100001, 0,
  'agent.expert.specialized-model-qa', 38465,
  NULL, 57, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000286, '6498e288-df18-4c07-8cbc-a06e26142fdb', 100001, 0,
  'agent.expert.specialized-pricing-analyst', 22820,
  NULL, 58, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000287, 'ab241b07-1b12-4ef1-8c8d-a72efbb36ce2', 100001, 0,
  'agent.expert.specialized-pricing-optimizer', 9403,
  NULL, 59, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000288, '8d9dccb7-80ab-46e3-85d0-e710a6303755', 100001, 0,
  'agent.expert.specialized-risk-assessor', 16101,
  'enterprise', 60, TIMESTAMPTZ '2026-11-18T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000289, '57bc98a1-c60c-4d20-8971-e734938876f9', 100001, 0,
  'agent.expert.specialized-salesforce-architect', 22764,
  NULL, 61, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000290, '5ac3fef9-cbc8-4fe3-8fe3-de9b9b3e0f54', 100001, 0,
  'agent.expert.specialized-strategy-duel-agent', 10386,
  NULL, 62, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000291, '8bf89060-cd15-42c3-8e9b-1d49b9e5ccd3', 100001, 0,
  'agent.expert.specialized-workflow-architect', 20789,
  NULL, 63, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000292, 'f4c4f2ed-cfff-46be-87d7-cc2de91b53af', 100001, 0,
  'agent.expert.study-abroad-advisor', 31253,
  NULL, 64, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000293, 'cc34351b-dd2f-41c4-8413-a69ca27abfef', 100001, 0,
  'agent.expert.supply-chain-garment-factory-planning-engineer', 31289,
  NULL, 0, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000294, 'b88c9a51-66b3-4a30-82d2-24bf144edb36', 100001, 0,
  'agent.expert.supply-chain-inventory-forecaster', 9805,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000295, 'ebbb4992-5490-4422-8585-5c22a878bf87', 100001, 0,
  'agent.expert.supply-chain-route-optimizer', 32995,
  NULL, 2, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000296, '8a709a3c-3145-4a6e-8749-3cde51ef4378', 100001, 0,
  'agent.expert.supply-chain-strategist', 38071,
  NULL, 65, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000297, '74f8ebf4-54c1-4773-8ddf-f582dbdc1cf1', 100001, 0,
  'agent.expert.supply-chain-vendor-evaluator', 4340,
  NULL, 3, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000298, '65b35a68-9d62-45ca-8da9-997161df7422', 100001, 0,
  'agent.expert.support-analytics-reporter', 37836,
  'leader', 0, TIMESTAMPTZ '2026-11-18T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000299, 'dc450b18-b76d-4341-8727-c3b993c62952', 100001, 0,
  'agent.expert.support-executive-summary-generator', 17071,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000300, 'a6674d86-9169-4621-8b3b-ac05ee839ca3', 100001, 0,
  'agent.expert.support-finance-tracker', 12020,
  NULL, 2, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000301, '15bcffc7-f1cb-4c90-8dfd-b487c497df44', 100001, 0,
  'agent.expert.support-infrastructure-maintainer', 2920,
  NULL, 3, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000302, '54ea3b1f-abc1-4cbe-8d3c-a526ea80c4c2', 100001, 0,
  'agent.expert.support-legal-compliance-checker', 39033,
  NULL, 4, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000303, '0a76c3f1-2b5a-4bec-84b1-2d361f105f31', 100001, 0,
  'agent.expert.support-recruitment-specialist', 16313,
  NULL, 5, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000304, '86017166-a28c-4396-8865-e15d5b80b423', 100001, 0,
  'agent.expert.support-support-responder', 11121,
  NULL, 6, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000305, 'c51a5e8f-4ac9-4cd8-8cd2-cac581c4ad02', 100001, 0,
  'agent.expert.technical-artist', 1674,
  NULL, 12, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000306, '62ed305b-040a-4ca0-849e-a9fadf0e5e2c', 100001, 0,
  'agent.expert.technical-translator-agent', 23548,
  NULL, 66, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000307, '3a3e48a9-8330-4653-83ff-f75739531de7', 100001, 0,
  'agent.expert.terminal-integration-specialist', 36312,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000308, '8ae4ad82-86aa-4d29-8df9-224b9b75751a', 100001, 0,
  'agent.expert.testing-accessibility-auditor', 32257,
  NULL, 0, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000309, '2bf6cd4e-699a-4265-8f3b-3294ff0433bf', 100001, 0,
  'agent.expert.testing-api-tester', 37654,
  NULL, 1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000310, 'd5412805-6adb-48b4-8de2-d4096d791aa2', 100001, 0,
  'agent.expert.testing-embedded-qa-engineer', 33746,
  NULL, 2, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000311, '8fa5a6da-a8d7-420a-865e-0f163a585cb4', 100001, 0,
  'agent.expert.testing-evidence-collector', 25079,
  NULL, 3, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000312, '11fe7759-ecf3-4fc9-877d-6f7305a0e5e9', 100001, 0,
  'agent.expert.testing-performance-benchmarker', 14952,
  NULL, 4, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000313, 'a1d2db85-c321-40a6-8cb4-830e7881bf59', 100001, 0,
  'agent.expert.testing-reality-checker', 37414,
  NULL, 5, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000314, 'e7c6fbce-644c-4769-8609-73589ee16624', 100001, 0,
  'agent.expert.testing-test-automation-engineer', 38816,
  NULL, 6, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000315, '2d056558-7e4f-4e13-843c-2c56bf1c22df', 100001, 0,
  'agent.expert.testing-test-results-analyzer', 18289,
  NULL, 7, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000316, '62eeec03-c2d6-4191-859c-1d86d21e1f38', 100001, 0,
  'agent.expert.testing-tool-evaluator', 5714,
  NULL, 8, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000317, '3db20b5d-455e-425d-8b40-de3d03f0f89b', 100001, 0,
  'agent.expert.testing-workflow-optimizer', 20779,
  NULL, 9, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000318, '4f483af2-ced2-467e-8d1a-4a2fc9e41082', 100001, 0,
  'agent.expert.travel-planner', 32243,
  NULL, 67, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000319, '7ccf0667-1912-4bf4-84b2-33df75edb207', 100001, 0,
  'agent.expert.unity-architect', 5480,
  NULL, 13, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000320, '0e40aa80-ccf0-452d-80a7-4f787f071014', 100001, 0,
  'agent.expert.unity-editor-tool-developer', 16494,
  NULL, 14, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000321, 'c5a30889-8796-44fd-8531-46549ae925d4', 100001, 0,
  'agent.expert.unity-multiplayer-engineer', 10137,
  NULL, 15, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000322, '39fda3e8-5471-4a6f-821d-465787ef5994', 100001, 0,
  'agent.expert.unity-shader-graph-artist', 39512,
  NULL, 16, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000323, '5b9d31f1-e58b-4939-8fba-097ea8a22dfb', 100001, 0,
  'agent.expert.unreal-multiplayer-architect', 7285,
  NULL, 17, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000324, '5f4bc101-8fcb-4b91-8e22-098b45f80dd7', 100001, 0,
  'agent.expert.unreal-systems-engineer', 32852,
  NULL, 18, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000325, 'a89fe485-11ad-4a49-8020-9987467fce64', 100001, 0,
  'agent.expert.unreal-technical-artist', 5355,
  NULL, 19, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000326, '9e7c39f5-2ed2-4117-8db0-d16c12b7dc09', 100001, 0,
  'agent.expert.unreal-world-builder', 23081,
  NULL, 20, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000327, '97c10f41-f2a5-4a1e-811d-4ba2a960d570', 100001, 0,
  'agent.expert.visionos-spatial-engineer', 35633,
  NULL, 2, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000328, 'aa7f56ec-9950-4725-8624-ddb5072ddb7d', 100001, 0,
  'agent.expert.xr-cockpit-interaction-specialist', 38267,
  NULL, 3, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000329, 'b94f3bfb-0f76-4d25-82bf-536185dd0a31', 100001, 0,
  'agent.expert.xr-immersive-developer', 41184,
  NULL, 4, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000330, 'd7d19599-21d7-4bba-821b-1d32777b58ff', 100001, 0,
  'agent.expert.xr-interface-architect', 8192,
  NULL, 5, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

INSERT INTO ai_expert_profile (
  id, uuid, tenant_id, organization_id, expert_agent_id, usage_count,
  badge_code, sort_order, published_at, created_by, updated_by, created_at,
  updated_at, deleted_at, deleted_by
) VALUES (
  373000000000000331, '144989f3-5205-4182-87aa-1012164cb5ad', 100001, 0,
  'agent.expert.zk-steward', 27729,
  NULL, 68, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', 1,
  1, TIMESTAMPTZ '2026-11-20T00:00:00+08:00', TIMESTAMPTZ '2026-11-20T00:00:00+08:00', NULL, NULL
)
ON CONFLICT (tenant_id, organization_id, expert_agent_id) DO UPDATE SET
  usage_count = EXCLUDED.usage_count,
  badge_code = EXCLUDED.badge_code,
  sort_order = EXCLUDED.sort_order,
  published_at = EXCLUDED.published_at,
  updated_by = EXCLUDED.updated_by,
  updated_at = EXCLUDED.updated_at,
  deleted_at = NULL
;

COMMIT;
