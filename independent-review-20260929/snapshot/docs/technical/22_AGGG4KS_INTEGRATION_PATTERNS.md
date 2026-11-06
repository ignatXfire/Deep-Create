# Deep Create - AGGG4ks Integration Patterns
# Extracted patterns from AGGG4ks for offline operation Deep Create

## 1. Spec-Driven Development (Kiro Methodology)

Source: AGGG4ks/AGGG4/agents/spec-planner.md

```
Описание фичи → .kiro/specs/<slug>/{requirements.md, design.md, tasks.md}
```

**Application in Deep Create:**
- Each WorkUnit starts with spec: requirements → design → tasks
- EARS notation: FR-x, NFR-x, AC-x.y Given/When/Then
- Tracing: FR → design → TSK-xxx
- Quality checklists before completion

## 2. PI Bootstrap Pattern

Source: AGGG4ks/AGGG4/skills/pi-bootstrap/SKILL.md

```
ОС детект → установка → 14 плагинов → ключи → MCP → гейты
```

**Application in Deep Create:**
- Automatic environment setup for agents
- Idempotent init scripts
- Isolation of keys and credentials

## 3. Agent Roles (AGGG4/agents/)

| Agent | Role | Deep Create equivalent |
|-------|------|------------------------|
| explore | Exploring the unknown | ResearchWorkUnit |
| plan | Planning by Kiro | PlanningWorkUnit |
| spec-planner | Spec-driven planning | SpecWorkUnit |
| spec-reviewer | Review Specifications | ReviewWorkUnit |
| verify | Verification | VerificationWorkUnit |

## 4. Harness Penetration

Source: AGGG4ks/AGGG4/harnesses/registry.json

12+ tools with rules, commands, agents, plugins.

**Application in Deep Create:**
- Agents work in different environments (Hermes, opencode, Claude Code, Cursor)
- Unified launch interface via harness registry
- Automatic adaptation to available tools

## 5. Firmware Update Pattern

Source: AGGG4ks/docs/update.md

```
--dry-run → показать план → ждать «применяй» → применить с бэкапом
```

**Application in Deep Create:**
- All changes go through dry-run preview
- Automatic backup before apply
- Rollback from backup in case of failure

## 6. Skills as Canon

Source: AGGG4ks/skills/

```
SKILL.md = источник правды → производные per-CLI через макросы
```

**Application in Deep Create:**
- Domain knowledge in SKILL.md files
- Automatic generation of prompts from skills
- Consistency between different agents

## 7. Red-First Testing

Source: AGGG4ks/AGGG4/AGENTS.md §3.5

```
Красное → verify exit 2 → CI FAIL
```

**Application in Deep Create:**
- VerificationWorkUnit should find the problem before merge
- No green build without passing verification

## 8. Worktree Isolation (from handoff document)

```
Каждый агент — свой git worktree + ветка
Проверка занятых ресурсов перед запуском
```

**Application in Deep Create:**
- dc-store manages isolated workspaces
- Resource registry: ports, database, Redis-prefix, Docker names
- Automatic conflict detection

## Integration into dc-protocol

New message types for Deep Create protocol:

```rust
pub enum DeepCreateCommand {
    // Spec-driven workflow
    CreateSpec { slug: String, description: String },
    ApproveSpec { slug: String },
    
    // Agent orchestration
    SpawnAgent { role: AgentRole, work_unit: UnitRef },
    ListAgents,
    
    // Worktree management
    CreateWorktree { unit: UnitRef },
    MergeWorktree { unit: UnitRef, strategy: MergeStrategy },
    
    // Verification
    RunVerification { unit: UnitRef },
    ReportVerification { unit: UnitRef, result: VerificationRecord },
}

pub enum AgentRole {
    Explorer,
    Planner,
    SpecWriter,
    SpecReviewer,
    Implementer,
    Verifier,
    Integrator,
}
```

## Next steps

1. Create dc-protocol message types for agent orchestration
2. Integrate worktree management into dc-store
3. Add spec-driven workflow to dc-domain
4. Create a harness registry for multi-environment support
