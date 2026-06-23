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

If an accepted ADR conflicts with implementation:
* **STOP.**
* Explain the conflict.
* Propose alternatives.
* Do not continue.
