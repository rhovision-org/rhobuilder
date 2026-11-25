/-!
Extracted from r-share (AGPL-3.0-or-later), spec/lean/RShare, commit 7114acba.
Namespace-generalized as RhoApp; theorem statements are unchanged.
-/
/-!
# Basic quantities

All amounts are integer minor units. Heights are block numbers. The contracts
use `Nat` where a guard supplies non-negativity.

The model is deliberately shallow: it does not formalise Rholang syntax, ERTP,
phlo or cryptography. The flow manifest in `spec/flows` ties each arm to a
declaration here; that is traceability, not a refinement proof.
-/

namespace RhoApp

abbrev Amount := Nat
abbrev Height := Nat
abbrev BlockSpan := Nat
abbrev Bps : Nat := 10000

/-- Truncating division, the node's `/` on naturals. -/
def floorDiv (n d : Nat) : Nat := n / d

/-- Division rounded up. `ceilDiv n 0` is `0` by Lean's convention and is never
reached: every caller guards `d > 0`. -/
def ceilDiv (n d : Nat) : Nat := (n + d - 1) / d

def minNat (a b : Nat) : Nat := if a ≤ b then a else b
def maxNat (a b : Nat) : Nat := if a ≤ b then b else a

def feeOf (rent bps : Nat) : Nat := rent * bps / Bps

/-- A truncated basis-point share of `amount`. -/
def share (amount bps : Nat) : Nat := amount * bps / Bps

/-- Sum of a list of naturals. -/
def sumNat : List Nat → Nat
  | [] => 0
  | x :: xs => x + sumNat xs

end RhoApp
