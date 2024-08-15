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

        // Process definitions
        proc1: $ => choice(
            seq("if", "(", $.proc, ")", $.proc2),
            seq("if", "(", $.proc, ")", $.proc2, "else", $.proc1),
            seq("new", repeat($.name_decl), "in", $.proc1),
            seq("contract", $.name, "(", repeat($.name), $.name_remainder, ")", "=", "{", $.proc, "}"),
            seq("let", $.decl, repeat($.decls), "in", "{", $.proc, "}")
        ),

        proc2: $ => choice(
            seq("for", "(", repeat($.receipt), ")", "{", $.proc, "}"),
            seq("select", "{", repeat($.branch), "}"),
            seq("match", $.proc4, "{", repeat($.case_impl), "}"),
            seq("bundle", $.bundle, "{", $.proc, "}")
        ),

        proc3: $ => seq($.name, $.send, "(", repeat($.proc), ")"),

        proc4: $ => seq($.proc4, "or", $.proc5),
        proc5: $ => seq($.proc5, "and", $.proc6),
        proc6: $ => choice(
            seq($.proc7, "matches", $.proc7),
            seq($.proc6, "==", $.proc7),
            seq($.proc6, "!=", $.proc7)
        ),

        proc7: $ => choice(
            seq($.proc7, "<", $.proc8),
            seq($.proc7, "<=", $.proc8),
            seq($.proc7, ">", $.proc8),
            seq($.proc7, ">=", $.proc8)
        ),

        proc8: $ => choice(
            seq($.proc8, "+", $.proc9),
            seq($.proc8, "-", $.proc9),
            seq($.proc8, "++", $.proc9),
            seq($.proc8, "--", $.proc9)
        ),

        proc9: $ => choice(
            seq($.proc9, "*", $.proc10),
            seq($.proc9, "/", $.proc10),
            seq($.proc9, "%", $.proc10),
            seq($.proc9, "%%", $.proc10)
        ),

        proc10: $ => choice(
            seq("not", $.proc10),
            seq("-", $.proc10)
        ),

        proc11: $ => choice(
            seq($.proc11, ".", $.var, "(", repeat($.proc), ")"),
            seq("(", $.proc4, ")")
        ),

        proc12: $ => seq("*", $.name),
        proc13: $ => seq($.proc13, "\\/", $.proc14),
        proc14: $ => seq($.proc14, "/\\", $.proc15),
        proc15: $ => seq("~", $.proc15),
        proc16: $ => choice($.ground, $.collection, $.proc_var, "Nil", $.simple_type),

        proc_block: $ => seq("{", $.proc, "}"),
        proc: $ => seq($.proc, "|", $.proc1),

        // Receipt definitions
        receipt: $ => choice(
            $.receipt_linear,
            $.receipt_repeated,
            $.receipt_peek
        ),

        receipt_linear: $ => seq(
            repeat($.linear_bind), optional($.proc)
        ),

        linear_bind: $ => seq(
            repeat($.name), optional($.name_remainder), "<-", $.name_source
        ),

        name_source: $ => choice(
            $.name,
            seq($.name, "?!"),
            seq($.name, "!?", "(", repeat($.proc), ")")
        ),

        receipt_repeated: $ => seq(
            repeat($.repeated_bind), optional($.proc)
        ),

        repeated_bind: $ => seq(
            repeat($.name), optional($.name_remainder), "<=", $.name
        ),

        receipt_peek: $ => seq(
            repeat($.peek_bind), optional($.proc)
        ),

        peek_bind: $ => seq(
            repeat($.name), optional($.name_remainder), "<<-", $.name
        ),

        // Branches
        branch: $ => seq($.receipt_linear, "=>", $.proc3),

        // Match cases
        case_impl: $ => seq($.proc13, "=>", $.proc3),

        // Remainders
        proc_remainder: $ => choice(
            seq("...", $.proc_var),
            ""
        ),

        // Names and variables
        name: $ => choice($.var, seq("@", $.proc12)),
        name_remainder: $ => choice(seq("...", "@", $.proc_var), ""),

        proc_var: $ => choice("_", $.var),

        name_decl: $ => choice(
            $.var,                       // Просте ім'я
            seq($.var, "(", $.uri_literal, ")")  // Ім'я з URI
        ),

        // Declarations
        decl: $ => seq(
            repeat($.name), $.name_remainder, "<-", repeat($.proc)
        ),

        decls: $ => choice(
            ";",    // Linear declaration
            "&"     // Concurrent declaration
        ),

        // Bundle
        bundle: $ => choice("bundle+", "bundle-", "bundle0", "bundle"),

        // Send types
        send: $ => choice("!", "!!"),

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
