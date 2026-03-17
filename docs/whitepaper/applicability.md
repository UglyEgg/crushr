# Applicability and roadmap

crushr is best understood as a serious archival utility for workflows that value verifiable post-damage recovery. It is not trying to be the default answer for every routine packaging task.

## Fit

crushr aligns well with archival and preservation workflows, corruption research, long-lived storage experiments, and any environment where the difference between “can no longer extract cleanly” and “can still prove and recover useful data” matters operationally.

## Current limits

The format is still in active development. The architecture is now strong enough to justify formal documentation and productization, but some features remain intentionally out of the core until the architecture is fully hardened. These include the broader POSIX metadata envelope, extended attributes, and later compression-dictionary refinements.

## Roadmap logic

The sequencing is deliberate.

1. Finish architecture hardening and namespace factoring.
2. De-cruft and separate canonical runtime paths from lab infrastructure.
3. Complete the bounded metadata envelope required for a real archival utility.
4. Improve compression intelligence without weakening the recovery model.

<div class="callout">
  <strong>Legitimacy does not require total completeness.</strong> A format becomes legitimate when it has a defensible purpose, a coherent architecture, a documented trust boundary, and an evaluation story that can survive professional scrutiny. crushr now has those ingredients.
</div>
