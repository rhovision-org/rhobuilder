import RhoApp.Basic

/-! Scaffold quantities over the vendored RhoApp library. This is not a Rholang refinement. -/

namespace __App__

abbrev Amount := RhoApp.Amount
abbrev Height := RhoApp.Height
abbrev Bps : Nat := RhoApp.Bps

def share (amount bps : Nat) : Nat := RhoApp.share amount bps

theorem scaffold_total : True := by trivial

end __App__
