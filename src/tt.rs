use crate::{types::Move, zobrist::ZKey};

#[inline(always)]
fn age_delta(current: u8, entry: u8) -> u8 {
    current.wrapping_sub(entry) & 0b1_1111
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum BoundType {
    None = 0,
    Lower = 0b01,
    Upper = 0b10,
    Exact = 0b11,
}

/// 8 bit struct representing node information for a TTEntry
///
/// bits 1-2: bound type
/// bits 3: pv node flag
/// bits 4-8: node age
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct NodeInfo(u8);

impl NodeInfo {
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline(always)]
    pub fn new(age: u8, pv_node: bool, bound: BoundType) -> Self {
        let mut info = 0u8;

        info |= bound as u8;
        info |= (pv_node as u8) << 2;
        info |= age << 3;

        Self(info)
    }

    #[inline(always)]
    pub fn age(&self) -> u8 {
        self.0 >> 3
    }

    #[inline(always)]
    pub fn is_pv(&self) -> bool {
        self.0 & 0b100 != 0
    }

    #[inline(always)]
    pub fn bound_type(&self) -> BoundType {
        match self.0 & 0b11 {
            0b00 => BoundType::None,
            0b01 => BoundType::Lower,
            0b10 => BoundType::Upper,
            0b11 => BoundType::Exact,
            _ => unreachable!(),
        }
    }
}

/// TTEntry struct is an 16 byte representation of a transposition table entry, defined as:
///
/// key: 64 bits \
/// depth: 8 bits \
/// age: 5 bits \
/// pv node: 1 bit \
/// bound type: 2 bits \
/// move: 16 bits \
/// value: 16 bits \
/// eval: 16 bits \
///
/// Note that value is the score the engine found during the search,
/// whereas eval represents the static evaluation of the position.
#[derive(Copy, Clone)]
pub struct TTEntry {
    key: ZKey,
    depth: u8,
    mv: Move,
    node_info: NodeInfo,
    value: i16,
    eval: i16,
}

impl TTEntry {
    pub const fn empty() -> Self {
        Self {
            key: ZKey(0),
            depth: 0,
            mv: Move::NULL,
            node_info: NodeInfo::empty(),
            value: 0,
            eval: 0,
        }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.key.0 == 0
    }

    #[inline(always)]
    pub fn new(key: ZKey, depth: u8, mv: Move, node_info: NodeInfo, value: i16, eval: i16) -> Self {
        Self {
            key,
            depth,
            mv,
            node_info,
            value,
            eval,
        }
    }

    #[inline(always)]
    pub fn update(&mut self, depth: u8, mv: Move, node_info: NodeInfo, value: i16, eval: i16) {
        self.depth = depth;
        self.mv = mv;
        self.node_info = node_info;
        self.value = value;
        self.eval = eval;
    }

    #[inline(always)]
    pub fn age(&self) -> u8 {
        self.node_info.age()
    }

    #[inline(always)]
    pub fn is_pv(&self) -> bool {
        self.node_info.is_pv()
    }

    #[inline(always)]
    pub fn bound_type(&self) -> BoundType {
        self.node_info.bound_type()
    }
}

/// Contains the data from the transposition table, just with the zobrist key omitted
pub struct TTProbe {
    pub depth: u8,
    pub mv: Move,
    pub node_info: NodeInfo,
    pub value: i16,
    pub eval: i16,
}

pub struct TranspositionTable {
    entries: Vec<TTEntry>,
    age: u8,
}

impl TranspositionTable {
    pub fn new(mb: usize) -> Self {
        // Find total amount of entries we can fit, rounded down to the nearest power of 2
        let bytes = mb.saturating_mul(1024 * 1024);
        let raw_count = bytes / std::mem::size_of::<TTEntry>();

        let count = match raw_count {
            0 => 1,
            n if n.is_power_of_two() => n,
            n => n.next_power_of_two() >> 1,
        };

        debug_assert!(count.is_power_of_two());

        Self {
            entries: vec![TTEntry::empty(); count],
            age: 0,
        }
    }

    #[inline(always)]
    fn index(&self, key: ZKey) -> usize {
        (key.0 as usize) & (self.entries.len() - 1)
    }

    pub fn clear(&mut self) {
        self.entries.fill(TTEntry::empty());
        self.age = 0;
    }

    pub fn increment_age(&mut self) {
        self.age = self.age.wrapping_add(1);
    }

    pub fn probe(&self, key: ZKey) -> Option<TTProbe> {
        let entry = &self.entries[self.index(key)];

        if entry.is_empty() || entry.key != key {
            return None;
        }

        Some(TTProbe {
            depth: entry.depth,
            mv: entry.mv,
            node_info: entry.node_info,
            value: entry.value,
            eval: entry.eval,
        })
    }

    pub fn store(
        &mut self,
        key: ZKey,
        depth: u8,
        mv: Move,
        bound: BoundType,
        is_pv: bool,
        value: i16,
        eval: i16,
    ) {
        let node_info = NodeInfo::new(self.age, is_pv, bound);

        let index = self.index(key);
        let entry = &mut self.entries[index];

        // Replace the less valuable entry
        if (bound == BoundType::Exact)
            || (key != entry.key)
            || (depth > entry.depth)
            || (age_delta(self.age, entry.age()) != 0)
        {
            entry.update(depth, mv, node_info, value, eval);
            entry.key = key;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{MoveFlag, Square};

    use super::*;

    fn assert_correct_node_info(node: NodeInfo, age: u8, is_pv: bool, bound: BoundType) {
        assert_eq!(node.age(), age);
        assert_eq!(node.is_pv(), is_pv);
        assert_eq!(node.bound_type(), bound);
    }

    fn colliding_keys(tt: &TranspositionTable) -> (ZKey, ZKey) {
        let k1 = ZKey(1);
        let k2 = ZKey(1 + tt.entries.len() as u64);

        assert_eq!(tt.index(k1), tt.index(k2));
        (k1, k2)
    }

    #[test]
    fn age_delta_is_correct() {
        // If the provided current age is less than the entry age, it should calculate
        // the age delta modulo 32, otherwise it behaves as regular subtraction
        assert_eq!(age_delta(3, 12), 23);
        assert_eq!(age_delta(12, 4), 8);
    }

    #[test]
    fn node_info_round_trips() {
        let node = NodeInfo::empty();
        assert_correct_node_info(node, 0, false, BoundType::None);

        let node = NodeInfo::new(12, true, BoundType::Exact);
        assert_correct_node_info(node, 12, true, BoundType::Exact);
    }

    #[test]
    fn tt_store_probe_round_trip() {
        let mut tt = TranspositionTable::new(1);
        let key = ZKey(1);

        // Probing an empty table returns None
        let probe = tt.probe(key);
        assert!(probe.is_none());

        // Probing an existing key returns the stored values
        let mv = Move::new(Square::E2, Square::E4, MoveFlag::DoublePush);
        tt.store(key, 12, mv, BoundType::Upper, true, 20, 10);

        let probe = tt.probe(key).expect("Expected a TT hit");

        assert_eq!(probe.mv, mv);
        assert_eq!(probe.depth, 12);
        assert_eq!(probe.value, 20);
        assert_eq!(probe.eval, 10);
        assert_eq!(probe.node_info.bound_type(), BoundType::Upper);
        assert_eq!(probe.node_info.is_pv(), true);
        assert_eq!(probe.node_info.age(), 0);
    }

    #[test]
    fn same_key_deeper_replaces() {
        let mut tt = TranspositionTable::new(1);
        let key = ZKey(1);

        #[rustfmt::skip]
        tt.store(key, 4, Move::new(Square::E2, Square::E3, MoveFlag::Quiet), BoundType::Lower, false, 10, 5);
        #[rustfmt::skip]
        tt.store(key, 5, Move::new(Square::E2, Square::E4, MoveFlag::DoublePush), BoundType::Lower, false, 21, 9);

        let hit = tt.probe(key).expect("Expected a TT hit");
        assert_eq!(hit.depth, 5);
        assert_eq!(hit.value, 21);
    }

    #[test]
    fn same_key_shallower_non_exact_does_not_replace() {
        let mut tt = TranspositionTable::new(1);
        let key = ZKey(1);
        let old_move = Move::new(Square::E2, Square::E3, MoveFlag::Quiet);
        let new_move = Move::new(Square::E2, Square::E4, MoveFlag::DoublePush);

        #[rustfmt::skip]
        tt.store(key, 6, old_move, BoundType::Lower, false, 10, 5);
        #[rustfmt::skip]
        tt.store(key, 3, new_move, BoundType::Lower, false, 21, 9);

        let hit = tt.probe(key).expect("Expected a TT hit");
        assert_eq!(hit.depth, 6);
        assert_eq!(hit.mv, old_move);
        assert_eq!(hit.value, 10);
    }

    #[test]
    fn exact_replaces_even_if_shallower() {
        let mut tt = TranspositionTable::new(1);
        let key = ZKey(1);

        #[rustfmt::skip]
        tt.store(key, 6, Move::new(Square::E2, Square::E3, MoveFlag::Quiet), BoundType::Lower, false, 10, 5);
        #[rustfmt::skip]
        tt.store(key, 3, Move::new(Square::E2, Square::E4, MoveFlag::DoublePush), BoundType::Exact, false, 21, 9);

        let hit = tt.probe(key).expect("Expected a TT hit");
        assert_eq!(hit.depth, 3);
        assert_eq!(hit.node_info.bound_type(), BoundType::Exact);
    }

    #[test]
    fn collision_replacement_matches_policy() {
        let mut tt = TranspositionTable::new(1);
        let (k1, k2) = colliding_keys(&tt);
        let old_move = Move::new(Square::E2, Square::E3, MoveFlag::Quiet);
        let new_move = Move::new(Square::E2, Square::E4, MoveFlag::DoublePush);

        #[rustfmt::skip]
        tt.store(k1, 6, old_move, BoundType::Lower, false, 10, 5);
        #[rustfmt::skip]
        tt.store(k2, 3, new_move, BoundType::Lower, false, 21, 9);

        // Different key collisions always replace
        let k1_probe = tt.probe(k1);
        assert!(k1_probe.is_none());

        let k2_probe = tt.probe(k2).expect("Expected a TT hit");
        assert_eq!(k2_probe.depth, 3);
        assert_eq!(k2_probe.mv, new_move);
        assert_eq!(k2_probe.value, 21);
    }

    #[test]
    fn older_entry_replaced_after_age_increment() {
        let mut tt = TranspositionTable::new(1);
        let key = ZKey(1);

        #[rustfmt::skip]
        tt.store(key, 6, Move::new(Square::E2, Square::E3, MoveFlag::Quiet), BoundType::Lower, false, 10, 5);
        tt.increment_age();
        #[rustfmt::skip]
        tt.store(key, 6, Move::new(Square::E2, Square::E4, MoveFlag::DoublePush), BoundType::Lower, false, 21, 9);

        let hit = tt.probe(key).expect("Expected a TT hit");
        assert_eq!(hit.value, 21);
        assert_eq!(hit.node_info.age(), 1);

        // Entry is replaced even after age wrap around
        for _ in 0..31 {
            tt.increment_age();
        }

        #[rustfmt::skip]
        tt.store(key, 7, Move::new(Square::E2, Square::E3, MoveFlag::Quiet), BoundType::Lower, false, 14, 7);

        let hit = tt.probe(key).expect("Expected a TT hit");
        assert_eq!(hit.value, 14);
        assert_eq!(hit.node_info.age(), 0);
    }
}
