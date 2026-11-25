#[path="../mod/src/set_workbench.rs"]mod set_workbench;
use set_workbench::*;
#[test]fn back_unwinds_subviews_then_returns_to_original_utility_section(){
 let mut n=Navigation::new();n.return_tab=4;n.screen=Screen::Settings;n.group=Some(2);n.renaming=true;
 assert_eq!(n.back(),Back::Stay);assert!(!n.renaming);assert_eq!(n.group,Some(2));
 assert_eq!(n.back(),Back::Stay);assert_eq!(n.group,None);
 assert_eq!(n.back(),Back::Stay);assert_eq!(n.screen,Screen::Search);
 assert_eq!(n.back(),Back::General(4));
 n.screen=Screen::SavedDetail;assert_eq!(n.back(),Back::Stay);assert_eq!(n.screen,Screen::Saved);
 assert_eq!(n.back(),Back::Stay);assert_eq!(n.screen,Screen::Search);
}
#[test]fn back_closes_dropdown_then_skill_list_then_priorities_then_settings(){
 let mut n=Navigation::new();n.screen=Screen::Settings;n.priorities=true;n.group=Some(0);n.dropdown=Some(4);
 assert_eq!(n.back(),Back::Stay);assert_eq!(n.dropdown,None);assert_eq!(n.group,Some(0));
 assert_eq!(n.back(),Back::Stay);assert!(n.priorities);assert_eq!(n.group,None);
 assert_eq!(n.back(),Back::Stay);assert!(!n.priorities);assert_eq!(n.screen,Screen::Settings);
 assert_eq!(n.back(),Back::Stay);assert_eq!(n.screen,Screen::Search);
}
