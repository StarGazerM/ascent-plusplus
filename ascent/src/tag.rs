struct RelationTag {
   id: usize,
   name: &'static str,
}

// a generic tag for a relation, it must has id and name, can construct from a vector of provenance tag
// trait ToRelTag {
//    fn to_rel_tag(&self, new_id: usize, new_name: &'static str, inputs: ) -> RelationTag;
// }
