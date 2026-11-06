#!/usr/bin/env python3
"""Minimal local control-plane skeleton.

This intentionally does NOT call any provider API. Replace run_agent() with your
Hermes/OpenCode/OmniRoute/DashScope runner. The point is deterministic admission,
authority locking, DAG scheduling, receipts, and backpressure-friendly queues.
"""
from __future__ import annotations
import asyncio, dataclasses, json, time, uuid
from collections import defaultdict

@dataclasses.dataclass
class Task:
    id: str
    objective: str
    deps: list[str]
    authority: list[str]
    model: str
    state: str = "SPEC_READY"

class ControlPlane:
    def __init__(self, max_writers: int = 16):
        self.tasks: dict[str, Task] = {}
        self.accepted: set[str] = set()
        self.locks: dict[str, str] = {}
        self.sem = asyncio.Semaphore(max_writers)
        self.events: list[dict] = []

    def add(self, task: Task):
        if task.id in self.tasks:
            raise ValueError(f"duplicate task {task.id}")
        self.tasks[task.id] = task

    def ready(self, t: Task) -> bool:
        return t.state == "SPEC_READY" and all(d in self.accepted for d in t.deps) and all(k not in self.locks for k in t.authority)

    def claim(self, t: Task):
        for k in t.authority:
            if k in self.locks:
                raise RuntimeError(f"authority conflict {k}")
        for k in t.authority:
            self.locks[k] = t.id
        t.state = "ADMITTED"
        self.emit("CLAIM", t.id, authority=t.authority)

    def release(self, t: Task):
        for k in list(t.authority):
            if self.locks.get(k) == t.id:
                del self.locks[k]

    def emit(self, kind, task_id, **data):
        self.events.append({"ts": time.time(), "kind": kind, "task_id": task_id, **data})

    async def run_agent(self, t: Task) -> dict:
        # TODO: invoke actual CLI/provider here and require an evidence receipt.
        await asyncio.sleep(0.05)
        return {
            "task_id": t.id,
            "attempt_id": str(uuid.uuid4()),
            "model": t.model,
            "status": "LOCAL_GREEN",
            "head_sha": "DEMO_HEAD_SHA",
            "commands": [{"command": "demo-test", "exit_code": 0, "tests_discovered": 1, "passed": 1, "failed": 0, "skipped": 0}],
            "artifacts": []
        }

    def validate_receipt(self, t: Task, r: dict) -> bool:
        if r.get("task_id") != t.id or r.get("status") != "LOCAL_GREEN":
            return False
        cmds = r.get("commands", [])
        return bool(cmds) and all(c.get("exit_code") == 0 and c.get("failed", 0) == 0 for c in cmds)

    async def execute(self, t: Task):
        async with self.sem:
            self.claim(t)
            t.state = "IMPLEMENTING"
            try:
                receipt = await self.run_agent(t)
                if not self.validate_receipt(t, receipt):
                    t.state = "FAIL"
                    self.emit("RECEIPT_REJECTED", t.id, receipt=receipt)
                    return
                # A real implementation should enqueue independent review here.
                t.state = "REVIEW_GREEN"
                self.accepted.add(t.id)
                self.emit("ACCEPTED_DEMO", t.id, receipt=receipt)
            except Exception as e:
                t.state = "FAIL"
                self.emit("RUN_FAILED", t.id, error=repr(e))
            finally:
                self.release(t)

    async def reconcile(self):
        running: set[asyncio.Task] = set()
        while True:
            for t in self.tasks.values():
                if self.ready(t):
                    t.state = "QUEUED"
                    async def launch(task=t):
                        task.state = "SPEC_READY"  # execute() performs atomic claim
                        await self.execute(task)
                    running.add(asyncio.create_task(launch()))
            if running:
                done, running = await asyncio.wait(running, timeout=0.1, return_when=asyncio.FIRST_COMPLETED)
            unresolved = [t for t in self.tasks.values() if t.state not in {"REVIEW_GREEN", "FAIL", "BLOCKED"}]
            if not unresolved and not running:
                break
            if not running and not any(self.ready(t) for t in unresolved):
                for t in unresolved:
                    t.state = "BLOCKED"
                    self.emit("BLOCKED", t.id, deps=t.deps)
                break
            await asyncio.sleep(0.02)

async def main():
    cp = ControlPlane(max_writers=4)
    cp.add(Task("WP-001", "Define shared API contract", [], ["api:user:v3"], "dash/qwen3.8-max-0902"))
    cp.add(Task("WP-002", "Implement user service", ["WP-001"], ["domain:user:service"], "dash/deepseek-v4-pro-0813"))
    cp.add(Task("WP-003", "Implement user UI", ["WP-001"], ["ui:user"], "dash/kimi-k3"))
    cp.add(Task("WP-004", "Integrate user journey", ["WP-002", "WP-003"], ["journey:user"], "dash/kimi-k3"))
    await cp.reconcile()
    print(json.dumps({"states": {k:v.state for k,v in cp.tasks.items()}, "events": cp.events}, indent=2))

if __name__ == "__main__":
    asyncio.run(main())
