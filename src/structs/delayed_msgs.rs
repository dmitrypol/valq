use crate::structs::valq_msg::ValqMsg;
use crate::utils;
use getset::Getters;
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Debug, Clone, Default, Getters)]
pub(crate) struct DelayedMsgs {
    #[getset(get = "pub")]
    scores: BTreeMap<u64, BTreeSet<ValqMsg>>, // Maps scores to message sets
    #[getset(get = "pub")]
    members: HashMap<ValqMsg, u64>, // Maps messages to their scores
}

impl DelayedMsgs {
    pub(crate) fn new() -> Self {
        Self {
            scores: BTreeMap::new(),
            members: HashMap::new(),
        }
    }

    pub(crate) fn insert(&mut self, member: ValqMsg, score: u64) {
        // Remove the member from its old score bucket if it exists
        match self.members.insert(member.clone(), score) {
            Some(old_score) => match self.scores.get_mut(&old_score) {
                Some(set) => {
                    set.remove(&member);
                    if set.is_empty() {
                        self.scores.remove(&old_score);
                    }
                }
                None => {}
            },
            None => {}
        }
        // Add the member to the new score bucket
        self.scores.entry(score).or_default().insert(member);
    }

    pub(crate) fn remove(&mut self, member: &ValqMsg) {
        match self.members.remove(member) {
            Some(score) => match self.scores.get_mut(&score) {
                Some(set) => {
                    set.remove(member);
                    if set.is_empty() {
                        self.scores.remove(&score);
                    }
                }
                None => {}
            },
            None => {}
        }
    }

    pub(crate) fn clear(&mut self) {
        self.scores.clear();
        self.members.clear();
    }

    pub(crate) fn len(&self) -> u64 {
        self.members.len() as u64
    }

    pub(crate) fn ready_to_process(&self) -> Vec<(u64, &ValqMsg)> {
        // get all members with their scores in the range of 0 to now
        let min = 0;
        let max = utils::now_as_seconds();
        self.scores
            .range(min..=max)
            .flat_map(|(score, members)| members.iter().map(|m| (*score, m)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::valq_msg::ValqMsg;

    #[test]
    fn test_insert_remove_len_clear() {
        let mut delayed_msgs = DelayedMsgs::new();
        let msg1 = ValqMsg::new(1, "message1".to_string(), None, 0);
        let msg2 = ValqMsg::new(2, "message2".to_string(), None, 0);

        delayed_msgs.insert(msg1.clone(), 100);
        assert_eq!(delayed_msgs.len(), 1);
        assert!(delayed_msgs.scores.contains_key(&100));
        assert!(delayed_msgs.members.contains_key(&msg1));

        delayed_msgs.insert(msg2.clone(), 200);
        assert_eq!(delayed_msgs.len(), 2);
        assert!(delayed_msgs.scores.contains_key(&200));
        assert!(delayed_msgs.members.contains_key(&msg2));

        delayed_msgs.remove(&msg1);
        assert_eq!(delayed_msgs.len(), 1);
        assert!(!delayed_msgs.members.contains_key(&msg1));

        delayed_msgs.clear();
        assert_eq!(delayed_msgs.len(), 0);
    }

    #[test]
    fn test_ready_to_process() {
        let mut delayed_msgs = DelayedMsgs::new();
        let msg1 = ValqMsg::new(1, "message1".to_string(), None, 0);
        let msg2 = ValqMsg::new(2, "message2".to_string(), None, 0);

        delayed_msgs.insert(msg1.clone(), utils::now_as_seconds());
        delayed_msgs.insert(msg2.clone(), utils::now_as_seconds() + 10);

        let ready = delayed_msgs.ready_to_process();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].1, &msg1);
    }
}
