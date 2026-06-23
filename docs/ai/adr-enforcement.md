# ADR Enforcement Policy

Before generating code:
1. **Read relevant ADRs.**
2. **Verify implementation matches ADR.**
3. **Report conflicts.**
4. **Refuse implementation that violates accepted ADRs.**

### Priority:
```
ADR > Rules > Code
```

## ADR Authority
Architecture Decision Records (ADR) are the highest technical authority.

Before making architectural decisions:
1. **Search ADRs.**
2. **Identify applicable ADRs.**
3. **Verify compatibility.**
4. **Cite ADRs used.**

*Never silently override an accepted ADR.*

If an accepted ADR conflicts with implementation:
* **STOP.**
* Explain the conflict.
* Suggest a migration path.
* Do not continue.

*No protocol code may violate accepted ADRs.*
