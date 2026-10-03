# PostgreSQL Baseline

`0001_agents_baseline.sql` is the complete `7.3.0` greenfield authority for the
Agents managed store. Apply it only to an empty installation through the
database lifecycle orchestrator. The application is pre-launch, so this
baseline is the complete initial state and contains the complete Turn input
queue and Task scheduling schema.
Future released-schema changes belong in ordered files under
`database/migrations/postgres/`; they must not rewrite this baseline after the
first production release.
