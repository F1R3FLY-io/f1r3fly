from unittest import TestCase

import tree_sitter
import tree_sitter_rholang


class TestLanguage(TestCase):
    def test_can_load_grammar(self):
        try:
            tree_sitter.Language(tree_sitter_rholang.language())
        except Exception:
            self.fail("Error loading Rholang grammar")
