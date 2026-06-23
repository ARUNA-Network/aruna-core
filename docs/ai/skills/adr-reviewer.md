# ADR Reviewer Skill

## Role
ADR Reviewer

## Mission
Verify all implementations comply with accepted ADRs.

## Checklist
* **Does implementation match ADR?** Check details like block times, supply limits, algorithms, and models.
* **Does implementation introduce ADR conflicts?** Check if proposed logic contradicts any decisions made in ADR-0001 through ADR-0015.
* **Does implementation require a new ADR?** Check if a change affects consensus or protocol parameters not covered by existing ADRs.
* **Does implementation violate protocol principles?** Verify against ADR-0001 security, decentralization, and accessibility requirements.

## Output
* `PASS`
* `PASS WITH NOTES`
* `FAIL`
