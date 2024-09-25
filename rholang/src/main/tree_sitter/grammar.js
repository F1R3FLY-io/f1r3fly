module.exports = grammar({
    name: 'rholang',
    word: $ => $.var,

    supertypes: $ => [
        $._decls,
        $._bundle,
        $._send_type,
        $._source,
        $._ground],

    inline: $ => [$.name, $.quotable],

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
            field('messages', alias($._proc_list, $.messages)),
            field('cont', $.sync_send_cont))
        ),
        empty_cont: $ => '.',
        non_empty_cont: $ => prec(1, seq(';', $._proc)),
        sync_send_cont: $ => choice($.empty_cont, $.non_empty_cont),

        new: $ => prec(1, seq(
            'new', field('decls', $.decls),
            'in',
            field('proc', $._proc))),
        decls: $ => commaSep1($.name_decl),

        ifElse: $ => prec.right(1, seq('if', '(', field('condition', $._proc), ')',
            field('ifTrue', $._proc),
            field('alternative', optional(seq('else', $._proc)))
        )),

        let: $ => prec(2, seq('let', field('decls', $._decls), 'in', field('proc', $.block))),

        bundle: $ => prec(2, seq(field('bundle_type', $._bundle), field('proc', $.block))),

        match: $ => prec(2, seq(
            'match',
            field('expression', $._proc_expression),
            '{', field('cases', $.cases), '}'
        )),
        cases: $ => repeat1($.case),

        choice: $ => prec(2, seq(
            'select',
            '{', field('branches', alias(repeat1($.branch), $.branches)), '}'
        )),

        contract: $ => prec(2, seq(
            'contract',
            field('name', $.name),
            '(', field('formals', $.names), ')',
            '=',
            field('proc', $.block)
        )),

        input: $ => prec(2, seq(
            'for',
            '(', field('formals', $.receipts), ')',
            field('proc', $.block)
        )),
        receipts: $ => semiSep1($._receipt),

        send: $ => prec(3, seq(
            field('name', $.name),
            field('send_type', $._send_type),
            field('inputs', alias($._proc_list, $.inputs))
        )),
        send_single: $ => '!',
        send_multiple: $ => '!!',
        _send_type: $ => choice($.send_single, $.send_multiple),

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
            field('args', alias($._proc_list, $.args)))
        ),

        eval: $ => prec(12, seq('*', $.name)),

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
        quote: $ => prec(12, seq('@', $.quotable)),

        name: $ => choice($.proc_var, $.quote),
        _name_remainder: $ => seq('...', '@', $.proc_var),
        names: $ => seq(commaSep1($.name), field('cont', optional($._name_remainder))),

        // let declarations
        decl: $ => seq(field('names', $.names), '<-', commaSep1($._proc)),
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
        linear_bind: $ => seq(
            field('names', $.names),
            '<-',
            field('input', $._source)
        ),

        repeated_bind: $ => seq(
            field('names', $.names),
            '<=',
            field('input', $.name)
        ),


        peek_bind: $ => seq(
            field('names', $.names),
            '<<-',
            field('input', $.name)
        ),

        _receipt: $ => choice(
            conc1($.linear_bind),
            conc1($.repeated_bind),
            conc1($.peek_bind)),

        simple_source: $ => $.name,
        receive_send_source: $ => seq($.name, '?!'),
        send_receive_source: $ => seq($.name, '!?', field('inputs', alias($._proc_list, $.inputs))),
        _source: $ => choice($.simple_source, $.receive_send_source, $.send_receive_source),

        // select branches
        branch: $ => seq(field('pattern', conc1($.linear_bind)), '=>', field('proc', choice($.send, $._proc_expression))),

        // name declarations
        name_decl: $ => seq($.var, field('uri', optional(seq('(', $.uri_literal, ')')))),

        //match cases
        case: $ => seq(field('pattern', $._proc), '=>', field('proc', $._proc)),

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
        collection: $ => choice($.list, $.tuple, $.set, $.map),

        list: $ => seq(
            '[',
            commaSep($._proc),
            field('cont', optional($._proc_remainder)),
            ']'
        ),

        set: $ => seq(
            'Set', '(',
            commaSep($._proc),
            field('cont', optional($._proc_remainder)),
            ')'
        ),

        map: $ => seq(
            '{',
            commaSep($.key_value_pair),
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

        _proc_list: $ => seq('(', commaSep($._proc), ')'),
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
