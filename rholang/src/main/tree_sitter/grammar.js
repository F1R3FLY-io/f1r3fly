module.exports = grammar({
    name: 'rholang',

    rules: {
        // Starting point of the grammar
        source_file: $ => repeat($._proc),

        _proc: $ => choice(
            $.proc1,
            $.proc2,
            $.proc3,
            $.proc4,
            $.proc5,
            $.proc6,
            $.proc7,
            $.proc8,
            $.proc9,
            $.proc10,
            $.proc11,
            $.proc12,
            $.proc13,
            $.proc14,
            $.proc15,
            $.proc16,
            $.proc_block
        ),

        proc1: $ => seq("if", "(", $.proc, ")", $.proc2),
        proc2: $ => choice(
            seq("new", repeat($.name_decl), "in", $.proc1),
            seq("contract", $.name, "(", repeat($.name), $.name_remainder, ")", "=", "{", $.proc, "}"),
            seq("for", "(", repeat($.receipt), ")", "{", $.proc, "}"),
            seq("select", "{", repeat($.branch), "}"),
            seq("match", $.proc4, "{", repeat($.case_impl), "}"),
            seq("bundle", $.bundle, "{", $.proc, "}"),
            seq("let", $.decl, repeat($.decls), "in", "{", $.proc, "}"),
            seq("select", "{" , repeat($.branch_impl), "}"),
            seq("match", $.proc4, "{", repeat($.case_impl), "}"),
            seq("bundle", $.bundle, "{", $.proc, "}"),
            seq("let", $.decl, repeat($.decls), "in", "{", $.proc, "}")
        ),

        proc_block: $ => seq("{", $.proc, "}"),
        proc16: $ => choice($.ground, $.collection, $.proc_var, "Nil", $.simple_type),
        proc15: $ => seq("~", $.proc15),
        proc14: $ => seq($.proc14, "/\\", $.proc15),
        proc13: $ => seq($.proc13, "\\/", $.proc14),
        proc12: $ => seq("*", $.name),
        proc11: $ => choice(
            seq($.proc11, ".", $.var, "(", repeat($.proc), ")"),
            seq("(", $.proc4, ")")
        ),
        proc10: $ => choice(
            seq("not", $.proc10),
            seq("-", $.proc10)
        ),
        proc9: $ => choice(
            seq($.proc9, "*", $.proc10),
            seq($.proc9, "/", $.proc10),
            seq($.proc9, "%", $.proc10),
            seq($.proc9, "%%", $.proc10)
        ),
        proc8: $ => choice(
            seq($.proc8, "+", $.proc9),
            seq($.proc8, "-", $.proc9),
            seq($.proc8, "++", $.proc9),
            seq($.proc8, "--", $.proc9)
        ),
        proc7: $ => choice(
            seq($.proc7, "<", $.proc8),
            seq($.proc7, "<=", $.proc8),
            seq($.proc7, ">", $.proc8),
            seq($.proc7, ">=", $.proc8)
        ),
        proc6: $ => choice(
            seq($.proc7, "matches", $.proc7),
            seq($.proc6, "==", $.proc7),
            seq($.proc6, "!=", $.proc7)
        ),
        proc5: $ => seq($.proc5, "and", $.proc6),
        proc4: $ => seq($.proc4, "or", $.proc5),
        proc3: $ => seq($.name, $.send, "(", repeat($.proc), ")"),
        proc2: $ => seq("for", "(", repeat($.receipt), ")", "{", $.proc, "}"),
        proc1: $ => choice(
            seq("if", "(", $.proc, ")", $.proc2),
            seq("if", "(", $.proc, ")", $.proc2, "else", $.proc1)
        ),
        proc: $ => seq($.proc, "|", $.proc1),

        // Names
        name: $ => choice($.var, seq("@", $.proc12)),

        // Bundle
        bundle: $ => choice("bundle+", "bundle-", "bundle0", "bundle"),

        // Send types
        send: $ => choice("!", "!!"),

        // Remainders
        proc_remainder: $ => choice("...", $.proc_var, ""),

        // Variables
        proc_var: $ => choice("_", $.var),
        name_remainder: $ => choice(seq("...", "@", $.proc_var), ""),

        // Ground types
        ground: $ => choice($.bool_literal, $.long_literal, $.string_literal, $.uri_literal),

        // Simple types
        simple_type: $ => choice("Bool", "Int", "String", "Uri", "ByteArray"),

        // Literals
        bool_literal: $ => choice("true", "false"),
        long_literal: $ => token(/\d+/),
        string_literal: $ => token(/"[^"\\]*(\\.[^"\\]*)*"/),
        uri_literal: $ => token(/`[^`\\]*(\\.[^`\\]*)*`/),

        // Variables
        var: $ => token(/((([a-zA-Z]|')|'_')([a-zA-Z]|[0-9]|'_'|'\')*)|(((_)([a-zA-Z]|[0-9]|'_'|'\')+))/),

        // Comments
        comment: $ => token(choice(
            seq('//', /.*/),
            seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/')
        )),

        // Collections
        collection: $ => choice(
            seq("[", repeat($.proc), $.proc_remainder, "]"),
            $.tuple,
            seq("Set", "(", repeat($.proc), $.proc_remainder, ")"),
            seq("{", repeat($.key_value_pair), $.proc_remainder, "}")
        ),

        // Key-Value Pair
        key_value_pair: $ => seq($.proc, ":", $.proc),

        // Tuple
        tuple: $ => choice(
            seq("(", $.proc, ",)"),
            seq("(", $.proc, ",", repeat($.proc), ")")
        ),
    }
});
