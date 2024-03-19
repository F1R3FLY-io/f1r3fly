use rspace_plus_plus::rspace::{
    matcher::r#match::Match,
    rspace::RSpace,
    state::{rspace_exporter::RSpaceExporter, rspace_importer::RSpaceImporter},
};
use serde::{Deserialize, Serialize};
use std::collections::LinkedList;

// See rspace/src/main/scala/coop/rchain/rspace/examples/StringExamples.scala
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
enum Pattern {
    #[default]
    Wildcard,
    StringMatch(String),
}

#[derive(Clone)]
struct StringMatch;

impl Match<Pattern, String> for StringMatch {
    fn get(&self, p: Pattern, a: String) -> Option<String> {
        match p {
            Pattern::Wildcard => Some(a),
            Pattern::StringMatch(value) => {
                if value == a {
                    Some(a)
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct StringsCaptor {
    res: LinkedList<Vec<String>>,
}

impl StringsCaptor {
    fn new() -> Self {
        StringsCaptor {
            res: LinkedList::new(),
        }
    }

    fn run_k(&mut self, data: Vec<String>) {
        self.res.push_back(data);
    }

    fn results(&self) -> Vec<Vec<String>> {
        self.res.iter().cloned().collect()
    }
}

// See rspace/src/test/scala/coop/rchain/rspace/ExportImportTests.scala
fn test_setup() -> (
    RSpace<String, Pattern, String, String, StringMatch>,
    Box<dyn RSpaceExporter>,
    Box<dyn RSpaceImporter>,
    RSpace<String, Pattern, String, String, StringMatch>,
    Box<dyn RSpaceExporter>,
    Box<dyn RSpaceImporter>,
) {
    todo!()
}
