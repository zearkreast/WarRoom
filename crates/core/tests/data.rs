mod common;
use common::*;
use warroom_core::data::{ListedReward, find_reward, reward_label, reward_text};

#[test]
fn names_known_rewards_and_falls_back_to_the_type_then_a_generic_label() {
    assert_eq!(reward_label(campaigns_live()[0].reward), "M-103 Supply FRV");
    assert_eq!(reward_label(Some(2659326411)), "Stratagem");
    assert_eq!(reward_label(None), "Campaign reward");
}

fn listed(mix_id: i64, kind: &str, name: Option<&str>) -> ListedReward {
    ListedReward { mix_id, kind: kind.to_string(), name: name.map(str::to_string) }
}

#[test]
fn a_listed_reward_names_one_the_built_in_list_lacks() {
    let rows = [listed(772085330, "Stratagem", Some("Flame FRV"))];
    assert_eq!(reward_text(Some(772085330), &rows), "Flame FRV");
    assert_eq!(reward_text(Some(772085330), &[]), "Campaign reward");
    assert_eq!(find_reward(772085330, &rows), Some(("Stratagem".to_string(), Some("Flame FRV".to_string()))));
}

#[test]
fn a_listed_reward_wins_over_the_built_in_list() {
    let rows = [listed(2659326411, "Stratagem", Some("Named later"))];
    assert_eq!(reward_text(Some(2659326411), &rows), "Named later");
    assert_eq!(reward_text(Some(2659326411), &[]), "Stratagem");
    assert_eq!(reward_text(Some(2768520552), &rows), "M-103 Supply FRV");
}
