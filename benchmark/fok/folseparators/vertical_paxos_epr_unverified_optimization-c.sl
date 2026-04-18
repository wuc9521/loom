(require "theories/vertical_paxos_epr_unverified_optimization-2bc0d415.sl")

(define-model P0
  (define-univ config (@config_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0))
  (define-univ value (@uc_value_0))
  (interp-fun master_minrel (@round_0))
  (interp-fun none (@round_0))
  (interp-fun quorum_of_round (match (
    ((@round_0) @quorum_0)
    ((_) @quorum_0)
  )))
  (interp-fun configure_round_msg (match (
    ((_ _) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun irrelevant_msg (match (
    ((_) false)
  )))
  (interp-fun joined_round (match (
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun minrelof (match (
    ((@config_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_ _) false)
  )))
  (interp-fun one_b_max_vote (match (
    ((_ _ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun quorumin (match (
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(define-model N1
  (define-univ config (@config_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@uc_value_0))
  (interp-fun master_minrel (@round_0))
  (interp-fun none (@round_0))
  (interp-fun quorum_of_round (match (
    ((@round_0) @quorum_0)
    ((@round_1) @quorum_0)
    ((_) @quorum_0)
  )))
  (interp-fun configure_round_msg (match (
    ((_ _) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun irrelevant_msg (match (
    ((_) false)
  )))
  (interp-fun joined_round (match (
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_1 @round_0) true)
    ((@round_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun minrelof (match (
    ((@config_0 @round_0) true)
    ((@config_0 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_ _) false)
  )))
  (interp-fun one_b_max_vote (match (
    ((_ _ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun quorumin (match (
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(define-model P2
  (define-univ config (@config_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@uc_value_0))
  (interp-fun master_minrel (@round_0))
  (interp-fun none (@round_1))
  (interp-fun quorum_of_round (match (
    ((@round_0) @quorum_0)
    ((@round_1) @quorum_0)
    ((_) @quorum_0)
  )))
  (interp-fun configure_round_msg (match (
    ((_ _) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun irrelevant_msg (match (
    ((_) false)
  )))
  (interp-fun joined_round (match (
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_0 @round_1) true)
    ((@round_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun minrelof (match (
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_ _) false)
  )))
  (interp-fun one_b_max_vote (match (
    ((_ _ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun quorumin (match (
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(define-model N3
  (define-univ config (@config_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@uc_value_0))
  (interp-fun master_minrel (@round_1))
  (interp-fun none (@round_0))
  (interp-fun quorum_of_round (match (
    ((@round_0) @quorum_0)
    ((@round_1) @quorum_0)
    ((_) @quorum_0)
  )))
  (interp-fun configure_round_msg (match (
    ((_ _) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun irrelevant_msg (match (
    ((_) false)
  )))
  (interp-fun joined_round (match (
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_1 @round_0) true)
    ((@round_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun minrelof (match (
    ((@config_0 @round_0) true)
    ((@config_0 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_ _) false)
  )))
  (interp-fun one_b_max_vote (match (
    ((_ _ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun quorumin (match (
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(define-model N4
  (define-univ config (@config_0 @config_1))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@uc_value_0))
  (interp-fun master_minrel (@round_1))
  (interp-fun none (@round_0))
  (interp-fun quorum_of_round (match (
    ((@round_0) @quorum_0)
    ((@round_1) @quorum_0)
    ((_) @quorum_0)
  )))
  (interp-fun configure_round_msg (match (
    ((_ _) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun irrelevant_msg (match (
    ((_) false)
  )))
  (interp-fun joined_round (match (
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_0 @round_1) true)
    ((@round_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun minrelof (match (
    ((@config_0 @round_0) true)
    ((@config_0 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_ _) false)
  )))
  (interp-fun one_b_max_vote (match (
    ((_ _ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun quorumin (match (
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(define-model P5
  (define-univ config (@config_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@uc_value_0))
  (interp-fun master_minrel (@round_1))
  (interp-fun none (@round_0))
  (interp-fun quorum_of_round (match (
    ((@round_0) @quorum_0)
    ((@round_1) @quorum_0)
    ((_) @quorum_0)
  )))
  (interp-fun configure_round_msg (match (
    ((_ _) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun irrelevant_msg (match (
    ((_) false)
  )))
  (interp-fun joined_round (match (
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_0 @round_1) true)
    ((@round_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun minrelof (match (
    ((@config_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_ _) false)
  )))
  (interp-fun one_b_max_vote (match (
    ((_ _ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun quorumin (match (
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(define-model P6
  (define-univ config (@config_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1))
  (define-univ value (@uc_value_0))
  (interp-fun master_minrel (@round_0))
  (interp-fun none (@round_1))
  (interp-fun quorum_of_round (match (
    ((@round_0) @quorum_0)
    ((@round_1) @quorum_0)
    ((_) @quorum_0)
  )))
  (interp-fun configure_round_msg (match (
    ((_ _) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun irrelevant_msg (match (
    ((_) false)
  )))
  (interp-fun joined_round (match (
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_0 @round_1) true)
    ((@round_1 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun minrelof (match (
    ((@config_0 @round_0) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_ _) false)
  )))
  (interp-fun one_b_max_vote (match (
    ((_ _ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun quorumin (match (
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(define-model N7
  (define-univ config (@config_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1 @round_2))
  (define-univ value (@uc_value_0))
  (interp-fun master_minrel (@round_0))
  (interp-fun none (@round_2))
  (interp-fun quorum_of_round (match (
    ((@round_0) @quorum_0)
    ((@round_1) @quorum_0)
    ((@round_2) @quorum_0)
    ((_) @quorum_0)
  )))
  (interp-fun configure_round_msg (match (
    ((_ _) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun irrelevant_msg (match (
    ((_) false)
  )))
  (interp-fun joined_round (match (
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_0 @round_1) true)
    ((@round_0 @round_2) true)
    ((@round_1 @round_1) true)
    ((@round_1 @round_2) true)
    ((@round_2 @round_2) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun minrelof (match (
    ((@config_0 @round_0) true)
    ((@config_0 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_ _) false)
  )))
  (interp-fun one_b_max_vote (match (
    ((_ _ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun quorumin (match (
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(define-model N8
  (define-univ config (@config_0))
  (define-univ node (@node_0))
  (define-univ quorum (@quorum_0))
  (define-univ round (@round_0 @round_1 @round_2))
  (define-univ value (@uc_value_0))
  (interp-fun master_minrel (@round_2))
  (interp-fun none (@round_0))
  (interp-fun quorum_of_round (match (
    ((@round_0) @quorum_0)
    ((@round_1) @quorum_0)
    ((@round_2) @quorum_0)
    ((_) @quorum_0)
  )))
  (interp-fun configure_round_msg (match (
    ((_ _) false)
  )))
  (interp-fun decision (match (
    ((_ _ _) false)
  )))
  (interp-fun irrelevant_msg (match (
    ((_) false)
  )))
  (interp-fun joined_round (match (
    ((_ _) false)
  )))
  (interp-fun le (match (
    ((@round_0 @round_0) true)
    ((@round_0 @round_1) true)
    ((@round_0 @round_2) true)
    ((@round_1 @round_1) true)
    ((@round_1 @round_2) true)
    ((@round_2 @round_2) true)
    ((_ _) false)
  )))
  (interp-fun left_rnd (match (
    ((_ _) false)
  )))
  (interp-fun member (match (
    ((@node_0 @quorum_0) true)
    ((_ _) false)
  )))
  (interp-fun minrelof (match (
    ((@config_0 @round_0) true)
    ((@config_0 @round_1) true)
    ((_ _) false)
  )))
  (interp-fun one_a (match (
    ((_ _) false)
  )))
  (interp-fun one_b_max_vote (match (
    ((_ _ _ _) false)
  )))
  (interp-fun proposal (match (
    ((_ _) false)
  )))
  (interp-fun quorumin (match (
    ((_ _) false)
  )))
  (interp-fun vote (match (
    ((_ _ _) false)
  )))
)

(synth-fun phi () Formula)

(constraint (FO.Sem phi (FoAsp P0 @config_0 @config_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0)))
(constraint (not (FO.Sem phi (FoAsp N1 @config_0 @config_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0))))
(constraint (FO.Sem phi (FoAsp P2 @config_0 @config_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0)))
(constraint (not (FO.Sem phi (FoAsp N3 @config_0 @config_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0))))
(constraint (not (FO.Sem phi (FoAsp N4 @config_0 @config_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0))))
(constraint (FO.Sem phi (FoAsp P5 @config_0 @config_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0)))
(constraint (FO.Sem phi (FoAsp P6 @config_0 @config_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0)))
(constraint (not (FO.Sem phi (FoAsp N7 @config_0 @config_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0))))
(constraint (not (FO.Sem phi (FoAsp N8 @config_0 @config_0 @node_0 @node_0 @quorum_0 @quorum_0 @round_0 @round_0 @uc_value_0 @uc_value_0))))

(check-synth)
