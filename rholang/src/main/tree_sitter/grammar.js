// module.exports = grammar({
//     name: 'rholang',
//
//     conflicts: $ => [
//         [$.proc4, $.proc7],
//         [$.name, $.proc_var]
//     ],
//
//     rules: {
//         // Starting point of the grammar
//         source_file: $ => repeat($._proc),
//
//         _proc: $ => choice(
//             $.proc16,
//             $.proc15,
//             $.proc14,
//             $.proc13,
//             $.proc12,
//             $.proc11,
//             $.proc10,
//             $.proc9,
//             $.proc8,
//             $.proc7,
//             $.proc6,
//             $.proc5,
//             $.proc4,
//             $.proc3,
//             $.proc2,
//             $.proc1,
//             $.proc_block
//         ),
//
//         // Process definitions
//         proc1: $ => prec.right(10, choice(
//             seq("new", repeat($.name_decl), "in", $._proc),
//             seq("match", $._proc, "{", repeat($.case_impl), "}"),
//             seq($.name, "!", "[", repeat($._proc), "]"),
//             seq("for", "(", repeat($.receipt), ")", "{", $._proc, "}")
//         )),
//
//         proc2: $ => prec.left(5, choice(
//             $.bundle_proc,
//             seq("contract", $.name, "(", repeat($.name), optional($.name_remainder), ")", "=", "{", $._proc, "}"),
//             seq("for", "(", repeat($.receipt), ")", "{", $._proc, "}"),
//             seq("select", "{", repeat($.branch), "}"),
//             seq("match", $._proc, "{", repeat($.case_impl), "}"),
//             seq("let", $.decl, repeat($.decls), "in", "{", $._proc, "}")
//         )),
//
//         proc3: $ => prec.left(1, choice(
//             seq($._proc, "&&", $._proc),
//             seq($._proc, "==", $._proc),
//             seq($._proc, "!=", $._proc)
//         )),
//
//         proc4: $ => prec.left(3, choice(
//             seq($._proc, "<", $._proc),
//             seq($._proc, "<=", $._proc),
//             seq($._proc, ">", $._proc),
//             seq($._proc, ">=", $._proc)
//         )),
//
//         proc5: $ => prec.left(seq($._proc, "|", $._proc)),
//
//         proc6: $ => prec.left(4, seq($._proc, "+", $._proc)),
//
//         proc7: $ => prec.left(3, choice(
//             seq($._proc, "<", $._proc),
//             seq($._proc, "<=", $._proc),
//             seq($._proc, ">", $._proc),
//             seq($._proc, ">=", $._proc)
//         )),
//
//         proc8: $ => prec.left(1, choice(
//             seq($._proc, "*", $._proc),
//             seq($._proc, "/", $._proc),
//             seq($._proc, "%", $._proc),
//             seq($._proc, "%%", $._proc)
//         )),
//
//         proc9: $ => prec.right(2, choice(
//             seq("-", $._proc),
//             seq("not", $._proc)
//         )),
//
//         proc10: $ => seq("*", $.name),
//
//         proc11: $ => seq($.name, ".", $.var, "(", repeat($._proc), ")"),
//
//         proc12: $ => prec.left(5, seq($._proc, "\\/", $._proc)),
//         proc13: $ => prec.left(6, seq($._proc, "/\\", $._proc)),
//         proc14: $ => prec.right(2, seq("~", $._proc)),
//         proc15: $ => choice(
//             $.ground,
//             // $.collection,
//             $.proc_var,
//             "Nil",
//             $.simple_type
//         ),
//
//         // proc16: $ => prec.right(3, choice(
//         //     seq("{", $._proc, "}"),
//         //     $.ground,
//         //     // $.collection,
//         //     $.proc_var,
//         //     "Nil",
//         //     $.simple_type
//         // )),
//
//         proc16: $ => prec.right(3, choice(
//             seq("{", $._proc, "}"),
//             $.ground,
//             //$.collection,
//             $.proc_var,
//             "Nil",
//             $.simple_type
//         )),
//
//         proc_block: $ => prec.right(2, seq("{", $._proc, "}")),
//
//         // Receipt definitions
//         receipt: $ => prec.left(seq(
//             repeat1($.linear_bind), optional($._proc)
//         )),
//
//         receipt_linear_impl: $ => seq(
//             repeat1($.linear_bind)
//         ),
//
//         linear_bind: $ => seq(
//             repeat($.name), optional($.name_remainder), "<-", $.name_source
//         ),
//
//         repeated_bind: $ => prec.left(seq(
//             repeat($.name), optional($.name_remainder), "<=", $.name
//         )),
//
//         peek_bind: $ => prec.left(seq(
//             repeat($.name), optional($.name_remainder), "<<-", $.name
//         )),
//
//         name_source: $ => choice(
//             $.name,
//             seq($.name, "?!"),
//             seq($.name, "!?", "(", repeat($._proc), ")")
//         ),
//
//         // Match cases
//         case_impl: $ => seq($._proc, "=>", $._proc),
//
//         // Remainders
//         proc_remainder: $ => choice(
//             seq("...", $.proc_var),
//             ""
//         ),
//
//         // Names and variables
//         name: $ => $.var,
//         name_remainder: $ => choice(
//             seq("...", "@", $.proc_var),
//             ""
//         ),
//
//         proc_var: $ => choice("_", $.var),
//
//         name_decl: $ => choice(
//             $.var,
//             seq($.var, "(", $.uri_literal, ")")
//         ),
//
//         // Bundle
//       bundle_proc: $ => choice(
//         prec(10, seq("bundle+", "{", $._proc, "}")),
//         prec(10, seq("bundle-", "{", $._proc, "}")),
//         prec(10, seq("bundle0", "{", $._proc, "}")),
//         prec(10, seq("bundle", "{", $._proc, "}"))
//       ),
//       // branch
//         branch: $ => seq(
//             $.receipt_linear_impl,
//             "=>",
//             $._proc
//         ),
//
//         // Define 'decls' as a sequence of linear or concurrent declarations
//         decls: $ => choice(
//             seq(";", repeat($.linear_decl)),
//             seq("&", repeat($.conc_decl))
//         ),
//
//         linear_decl: $ => $.decl,
//         conc_decl: $ => $.decl,
//
//         decl: $ => prec.left(seq(
//             repeat($.name), optional($.name_remainder), "<-", repeat($._proc)
//         )),
//
//         // Send types
//         send: $ => choice("!", "!!"),
//
//         // Ground types
//         ground: $ => choice($.bool_literal, $.long_literal, $.string_literal, $.uri_literal),
//
//         // Simple types
//         simple_type: $ => choice("Bool", "Int", "String", "Uri", "ByteArray"),
//
//         // Literals
//         bool_literal: $ => choice("true", "false"),
//         long_literal: $ => token(/\d+/),
//         string_literal: $ => token(/"[^"\\]*(\\.[^"\\]*)*"/),
//         uri_literal: $ => token(/`([^\\`]|\\.)*`/),
//
//         // Variables
//         var: $ => token(/((([a-zA-Z]|')|'_')([a-zA-Z]|[0-9]|'_'|'\')*)|(((_)([a-zA-Z]|[0-9]|'_'|'\')+))/),
//
//         // Comments
//         comment: $ => token(choice(
//             seq('//', /.*/),
//             seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/')
//         )),
//
//         // Collections
//         // collection: $ => choice(
//         //     seq("[", repeat($._proc), $.proc_remainder, "]"),
//         //     $.tuple,
//         //     seq("Set", "(", repeat($._proc), $.proc_remainder, ")"),
//         //     seq("{", repeat($.key_value_pair), $.proc_remainder, "}")
//         // ),
//
//         // Key-Value Pair
//         key_value_pair: $ => seq($._proc, ":", $._proc),
//
//         // Tuple
//         tuple: $ => choice(
//             seq("(", $._proc, ",)"),
//             seq("(", $._proc, ",", repeat($._proc), ")")
//         ),
//     }
// });

module.exports = grammar({
  name: 'rholang',
  word: $ => $.var,

  supertypes: $ => [$._decls, $._bundle, $._receipt, $._source, $._ground],

  inline: $ => [$.name, $.names, $.quotable],

  rules: {
    // Starting point of the grammar
    source_file: $ => repeat($._proc),

    _proc: $ => choice(
      $.par,
      $.send_sync,
      $.new,
      $.ifElse,
      $.let,
      $.bundle,
      $.match,
      $.choice,
      $.contract,
      $.input,
      $.send,
      $._proc_expression
    ),

    par: $ => prec.left(0, seq(field('left', $._proc), '|', field('right', $._proc))),

    send_sync: $ => prec(1, seq(
      field('name', $.name),
      '!?',
      '(', field('messages', commaSep($._proc)), ')',
      field('cont', $.sync_send_cont))
    ),
    empty_cont: $ => '.',
    non_empty_cont: $ => prec(1, seq(';', $._proc)),
    sync_send_cont: $ => choice($.empty_cont, $.non_empty_cont),

    new: $ => prec(1, seq('new', field('decls', commaSep1($.name_decl)), 'in', field('proc', $._proc))),

    ifElse: $ => prec.right(1, seq('if', '(', field('condition', $._proc), ')',
      field('ifTrue', $._proc),
      field('alternative', optional(seq('else', $._proc)))
    )),

    let: $ => prec(2, seq('let', field('decls', $._decls), 'in', field('proc', $.block))),

    bundle: $ => prec(2, seq(field('bundle', $._bundle), field('proc', $.block))),

    match: $ => prec(2, seq(
      'match',
      field('pattern', $._proc_expression),
      '{', field('cases', repeat1($.case)), '}'
    )),

    choice: $ => prec(2, seq(
      'select',
      '{', field('branches', repeat1($.branch)), '}'
    )),

    contract: $ => prec(2, seq(
      'contract',
      field('name', $.name),
      '(', field('formals', commaSep($.name)), field('cont', optional($._name_remainder)), ')',
      '=',
      field('proc', $.block)
    )),

    input: $ => prec(2, seq(
      'for',
      '(', field('formals', semiSep1($._receipt)), ')',
      field('proc', $.block)
    )),

    send: $ => prec(3, seq(
      field('name', $.name),
      field('send_type', choice('!', '!!')),
      '(', field('messages', commaSep($._proc)), ')'
    )),

    _proc_expression: $ => choice(
      $.or,
      $.and,
      $.matches,
      $.eq,
      $.neq,
      $.lt,
      $.lte,
      $.gt,
      $.gte,
      $.add,
      $.minus,
      $.plus_plus,
      $.minus_minus,
      $.percent_percent,
      $.mult,
      $.div,
      $.not,
      $.neg,
      $.method,
      $._parenthesized,
      $.eval,
      $.quote,
      $.disjunction,
      $.conjunction,
      $.negation,
      $._ground_expression
    ),
    _parenthesized: $ => prec(11, seq('(', $._proc_expression, ')')),

    or: $ => prec.left(4, seq(field('left', $._proc), 'or', field('right', $._proc))),
    and: $ => prec.left(5, seq(field('left', $._proc), 'and', field('right', $._proc))),
    matches: $ => prec.right(6, seq(field('left', $._proc), 'matches', field('right', $._proc))),
    eq: $ => prec.left(6, seq(field('left', $._proc), '==', field('right', $._proc))),
    neq: $ => prec.left(6, seq(field('left', $._proc), '!=', field('right', $._proc))),
    lt: $ => prec.left(7, seq(field('left', $._proc), '<', field('right', $._proc))),
    lte: $ => prec.left(7, seq(field('left', $._proc), '<=', field('right', $._proc))),
    gt: $ => prec.left(7, seq(field('left', $._proc), '>', field('right', $._proc))),
    gte: $ => prec.left(7, seq(field('left', $._proc), '>=', field('right', $._proc))),
    plus_plus: $ => prec.left(8, seq(field('left', $._proc), '++', field('right', $._proc))),
    minus_minus: $ => prec.left(8, seq(field('left', $._proc), '--', field('right', $._proc))),
    minus: $ => prec.left(8, seq(field('left', $._proc), '-', field('right', $._proc))),
    add: $ => prec.left(8, seq(field('left', $._proc), '+', field('right', $._proc))),
    percent_percent: $ => prec.left(9, seq(field('left', $._proc), '%%', field('right', $._proc))),
    mult: $ => prec.left(9, seq(field('left', $._proc), '*', field('right', $._proc))),
    div: $ => prec.left(9, seq(field('left', $._proc), '/', field('right', $._proc))),
    mod: $ => prec.left(9, seq(field('left', $._proc), '%', field('right', $._proc))),
    not: $ => prec(10, seq('not', field('proc', $._proc))),
    neg: $ => prec(10, seq('-', field('proc', $._proc))),

    method: $ => prec(11, seq(
      field('receiver', $._proc),
      '.',
      field('name', $.var),
      '(', field('args', commaSep($._proc)), ')')
    ),

    eval: $ => prec(12, seq('*', field('name', $.name))),

    disjunction: $ => prec.left(13, seq(field('left', $._proc), '\\/', field('right', $._proc))),
    conjunction: $ => prec.left(14, seq(field('left', $._proc), '/\\', field('right', $._proc))),
    negation: $ => prec(15, seq('~', field('proc', $._proc))),

    _ground_expression: $ => prec(16, choice($.block, $._ground, $.collection, $.proc_var, $.simple_type)),
    block: $ => seq('{', field('body', $._proc), '}'),

    // process variables and names
    wildcard: $ => '_',
    proc_var: $ => choice($.wildcard, $.var),

    quotable: $ => choice(
      $.eval,
      $.disjunction,
      $.conjunction,
      $.negation,
      $._ground_expression),
    quote: $ => prec(12, seq('@', field('proc', $.quotable))),

    name: $ => choice($.proc_var, $.quote),
    _name_remainder: $ => seq('...', '@', $.proc_var),
    names: $ => seq(field('names', commaSep1($.name)), field('cont', optional($._name_remainder))),

    // let declarations
    decl: $ => seq($.names, '<-', commaSep1($._proc)),
    linear_decls: $ => semiSep1($.decl),
    conc_decls: $ => prec(-1, conc1($.decl)),
    _decls: $ => choice($.linear_decls, $.conc_decls),

    // bundle
    bundle_write: $ => 'bundle+',
    bundle_read: $ => 'bundle-',
    bundle_equiv: $ => 'bundle0',
    bundle_read_write: $ => 'bundle',
    _bundle: $ => choice($.bundle_write, $.bundle_read, $.bundle_equiv, $.bundle_read_write),

    // receipts
    linear_receipt: $ => conc1($.linear_bind),
    linear_bind: $ => seq(
      $.names,
      '<-',
      field('input', $._source)
    ),

    repeated_receit: $ => conc1($.repeated_bind),
    repeated_bind: $ => seq(
      $.names,
      '<=',
      field('input', $.name)
    ),


    peek_receipt: $ => conc1($.peek_bind),
    peek_bind: $ => seq(
      $.names,
      '<<-',
      field('input', $.name)
    ),

    _receipt: $ => choice($.linear_receipt, $.repeated_receit, $.peek_receipt),

    simple_source: $ => field('name', $.name),
    receive_send_source: $ => seq(field('name', $.name), '?!'),
    send_receive_source: $ => seq(field('name', $.name), '!?', '(', field('inputs', commaSep($._proc)), ')'),
    _source: $ => choice($.simple_source, $.receive_send_source, $.send_receive_source),

    // select branches
    branch: $ => seq(field('pattern', $.linear_receipt), '=>', field('proc', choice($.send, $._proc_expression))),

    // name declarations
    name_decl: $ => seq($.var, field('uri', optional(seq('(', $.uri_literal, ')')))),

    //match cases
    case: $ => seq(field('pattern', $.quotable), '=>', field('proc', $._proc)),

    // ground terms
    nil: $ => 'Nil',
    _ground: $ => choice($.bool_literal, $.long_literal, $.string_literal, $.uri_literal, $.nil),

    // Literals
    bool_literal: $ => choice('true', 'false'),
    long_literal: $ => token(/\d+/),
    string_literal: $ => token(/"[^"\\]*(\\.[^"\\]*)*"/),
    uri_literal: $ => token(/`([^\\`]|\\.)*`/),

    // Simple types
    simple_type: $ => choice('Bool', 'Int', 'String', 'Uri', 'ByteArray'),

    // Collections
    collection: $ => choice($.collection_list, $.tuple, $.collection_set, $.collection_map),

    collection_list: $ => seq(
      '[',
      field('body', commaSep($._proc)),
      field('cont', optional($._proc_remainder)),
      ']'
    ),

    collection_set: $ => seq(
      'Set', '(',
      field('body', commaSep($._proc)),
      field('cont', optional($._proc_remainder)),
      ')'
    ),

    collection_map: $ => seq(
      '{',
      field('body', commaSep($.key_value_pair)),
      field('cont', optional($._proc_remainder)),
      '}'
    ),

    _proc_remainder: $ => seq('...', $.proc_var),

    key_value_pair: $ => seq(
      field('key', $._proc),
      ':',
      field('value', $._proc)
    ),

    tuple: $ => choice(
      seq('(', $._proc, ',)'),
      seq('(', commaSep1($._proc), ')')
    ),

    var: $ => token(/((([a-zA-Z]|')|'_')([a-zA-Z]|[0-9]|'_'|'\')*)|(((_)([a-zA-Z]|[0-9]|'_'|'\')+))/),
  }
});

function commaSep1(rule) {
  return seq(rule, repeat(seq(',', rule)))
}

function commaSep(rule) {
  return optional(commaSep1(rule))
}

function semiSep1(rule) {
  return seq(rule, repeat(seq(';', rule)))
}

function semiSep(rule) {
  return optional(semiSep1(rule))
}

function conc1(rule) {
  return seq(rule, repeat(seq('&', rule)))
}

function conc(rule) {
  return optional(conc1(rule))
}
