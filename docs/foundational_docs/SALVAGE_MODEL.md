# Salvage model

crushr treats recovery as a classification problem rather than a binary success/failure event.

<div class="figure tight">
  <img src="../assets/diagrams/recovery_pipeline.svg" alt="recovery pipeline" />
</div>

## Recovery classes

| Class | Meaning |
|---|---|
| Named recovery | Structural and naming proof both survive |
| Anonymous recovery | Structural proof survives but naming proof does not |
| Partial ordered recovery | Some extents missing, ordering still provable |
| Partial unordered recovery | Fragments survive but ordering is no longer provable |
| Orphan evidence | Verified fragments remain but cannot be reconstructed as files |
| No verified evidence | Nothing usable remains |

## Why anonymous fallback exists

Anonymous fallback is the mechanism that keeps crushr honest. If naming proof disappears, the format preserves verified payload without pretending it still knows the original path.

<div class="thesis">
  <strong>Short rule.</strong> Payload without names is acceptable. Names without proof are not.
</div>
