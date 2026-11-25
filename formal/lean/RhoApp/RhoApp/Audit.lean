import RhoApp.Basic
import RhoApp.Linear.Cell
import RhoApp.Linear.Protocol
import RhoApp.Linear.Deploy
import RhoApp.Linear.Concurrency

/-! Axiom audit for the shared linear-cell library. Every theorem is model-level. -/

#print axioms RhoApp.commit_all_or_none
#print axioms RhoApp.commit_preserves_wf
#print axioms RhoApp.run_failure_no_leak
#print axioms RhoApp.deploy_atomic_rollback
#print axioms RhoApp.h2a_caller_token
#print axioms RhoApp.commit_commute
#print axioms RhoApp.run_disjoint_commute
#print axioms RhoApp.disjoint_cells_linearizable
