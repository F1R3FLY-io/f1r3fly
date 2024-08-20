module.exports = grammar({
    name: 'rholang',

    conflicts: $ => [
        [$.proc4, $.proc7],
        [$.name, $.proc_var]
    ],

    rules: {
        // Starting point of the grammar
        source_file: $ => repeat($._proc),

        _proc: $ => choice(
            $.proc16,
            $.proc15,
            $.proc14,
            $.proc13,
            $.proc12,
            $.proc11,
            $.proc10,
            $.proc9,
            $.proc8,
            $.proc7,
            $.proc6,
            $.proc5,
            $.proc4,
            $.proc3,
            $.proc2,
            $.proc1,
            $.proc_block
        ),

        // Process definitions
        proc1: $ => prec.right(0, choice(
            seq("new", repeat($.name_decl), "in", $._proc),
            seq("match", $._proc, "{", repeat($.case_impl), "}"),
            seq($.name, "!", "[", repeat($._proc), "]"),
            seq("for", "(", repeat($.receipt), ")", "{", $._proc, "}")
        )),

        proc2: $ => prec.left(1, choice(
            seq("if", $._proc, "then", $._proc, "else", $._proc),
            seq($._proc, "||", $._proc)
        )),

        proc3: $ => prec.left(1, choice(
            seq($._proc, "&&", $._proc),
            seq($._proc, "==", $._proc),
            seq($._proc, "!=", $._proc)
        )),

        proc4: $ => prec.left(3, choice(
            seq($._proc, "<", $._proc),
            seq($._proc, "<=", $._proc),
            seq($._proc, ">", $._proc),
            seq($._proc, ">=", $._proc)
        )),

        proc5: $ => prec.left(seq($._proc, "|", $._proc)),

        proc6: $ => prec.left(4, seq($._proc, "+", $._proc)),

        proc7: $ => prec.left(3, choice(
            seq($._proc, "<", $._proc),
            seq($._proc, "<=", $._proc),
            seq($._proc, ">", $._proc),
            seq($._proc, ">=", $._proc)
        )),

        proc8: $ => prec.left(1, choice(
            seq($._proc, "*", $._proc),
            seq($._proc, "/", $._proc),
            seq($._proc, "%", $._proc),
            seq($._proc, "%%", $._proc)
        )),

        proc9: $ => prec.right(2, choice(
            seq("-", $._proc),
            seq("not", $._proc)
        )),

        proc10: $ => seq("*", $.name),

        proc11: $ => seq($.name, ".", $.var, "(", repeat($._proc), ")"),

        proc12: $ => prec.left(5, seq($._proc, "\\/", $._proc)),
        proc13: $ => prec.left(6, seq($._proc, "/\\", $._proc)),
        proc14: $ => prec.right(2, seq("~", $._proc)),
        proc15: $ => choice(
            $.ground,
            $.collection,
            $.proc_var,
            "Nil",
            $.simple_type
        ),

        proc16: $ => prec.right(3, choice(
            seq("{", $._proc, "}"),
            $.ground,
            $.collection,
            $.proc_var,
            "Nil",
            $.simple_type
        )),

        proc_block: $ => prec.right(2, seq("{", $._proc, "}")),

        // Receipt definitions
        receipt: $ => prec.left(seq(
            repeat1($.linear_bind), optional($._proc)
        )),

        linear_bind: $ => seq(
            repeat($.name), optional($.name_remainder), "<-", $.name_source
        ),

        repeated_bind: $ => prec.left(seq(
            repeat($.name), optional($.name_remainder), "<=", $.name
        )),

        peek_bind: $ => prec.left(seq(
            repeat($.name), optional($.name_remainder), "<<-", $.name
        )),

        name_source: $ => choice(
            $.name,
            seq($.name, "?!"),
            seq($.name, "!?", "(", repeat($._proc), ")")
        ),

        // Match cases
        case_impl: $ => seq($._proc, "=>", $._proc),

        // Remainders
        proc_remainder: $ => choice(
            seq("...", $.proc_var),
            ""
        ),

        // Names and variables
        name: $ => $.var,
        name_remainder: $ => choice(
            seq("...", "@", $.proc_var),
            ""
        ),

        proc_var: $ => choice("_", $.var),

        name_decl: $ => choice(
            $.var,
            seq($.var, "(", $.uri_literal, ")")
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
        uri_literal: $ => token(/[^\\]*(\\.[^\\]*)*/),

        // Variables
        var: $ => token(/((([a-zA-Z]|')|'_')([a-zA-Z]|[0-9]|'_'|'\')*)|(((_)([a-zA-Z]|[0-9]|'_'|'\')+))/),

        // Comments
        comment: $ => token(choice(
            seq('//', /.*/),
            seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/')
        )),

        // Collections
        collection: $ => choice(
            seq("[", repeat($._proc), $.proc_remainder, "]"),
            $.tuple,
            seq("Set", "(", repeat($._proc), $.proc_remainder, ")"),
            seq("{", repeat($.key_value_pair), $.proc_remainder, "}")
        ),

        // Key-Value Pair
        key_value_pair: $ => seq($._proc, ":", $._proc),

        // Tuple
        tuple: $ => choice(
            seq("(", $._proc, ",)"),
            seq("(", $._proc, ",", repeat($._proc), ")")
        ),
    }
});