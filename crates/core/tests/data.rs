mod common;
use common::*;
use warroom_core::data::reward_label;

#[test]
fn names_known_rewards_and_falls_back_to_the_type_then_a_generic_label() {
    assert_eq!(reward_label(campaigns_live()[0].reward), "M-103 Supply FRV");
    assert_eq!(reward_label(Some(2659326411)), "Stratagem");
    assert_eq!(reward_label(None), "Campaign reward");
}
