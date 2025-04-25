package tree_sitter_rholang_test

import (
	"testing"

	tree_sitter "github.com/tree-sitter/go-tree-sitter"
	tree_sitter_rholang "github.com/tree-sitter/tree-sitter-rholang/bindings/go"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_rholang.Language())
	if language == nil {
		t.Errorf("Error loading Rholang grammar")
	}
}
