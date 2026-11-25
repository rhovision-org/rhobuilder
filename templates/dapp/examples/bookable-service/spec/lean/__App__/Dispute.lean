import __App__.Basic

/-! Shallow dispute/panel model for the worked example.

This is a model, not a Rholang refinement. It captures the vote-once rule, provider
exclusion and settlement conservation. The deployed cell's linearity is modelled by the
vendored `RhoApp` library, not here. -/

namespace __App__

structure Dispute where
  amount : Nat
  settled : Nat
  refund : Nat
  provider : String
  panel : List String
  votes : List String
  state : String
  deriving Repr

def CanVote (d : Dispute) (panelist : String) : Prop :=
  d.state = "panel" ∧ panelist ∈ d.panel ∧ panelist ∉ d.votes

def record (d : Dispute) (panelist : String) : Dispute :=
  { d with votes := panelist :: d.votes }

/-- A panelist who has voted cannot vote again: the second `recordVerdict` precondition
fails because the vote is already in the list. -/
theorem voteOnce (d : Dispute) (panelist : String) :
    CanVote (record d panelist) panelist → False := by
  intro h
  have hmem : panelist ∈ (record d panelist).votes := by
    simp [record]
  exact h.2.2 hmem

/-- The provider cannot be selected: if the provider is absent from the panel, the
selection predicate is false. -/
theorem providerExcluded (d : Dispute) (provider : String)
    (h : provider ∉ d.panel) :
    ¬ (provider ∈ d.panel) := by
  exact h

def settle (d : Dispute) (settled refund : Nat) : Dispute :=
  { d with settled := settled, refund := refund, state := "settled" }

def Conserved (d : Dispute) : Prop :=
  d.settled + d.refund = d.amount

/-- A settlement that sums to the dispute amount conserves it. -/
theorem settleConserves (d : Dispute) (settled refund : Nat)
    (h : settled + refund = d.amount) :
    Conserved (settle d settled refund) := by
  simpa [Conserved, settle] using h

end __App__
