/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const PREC = {
  lowest: 0,
  assign: 1,
  or: 2,
  and: 3,
  equals: 4,
  less_greater: 5,
  sum: 6,
  product: 7,
  prefix: 8,
  call_index: 9,
};

const terminator = choice("\n", "\0");

module.exports = grammar({
  name: "aoc",

  word: ($) => $.identifier,

  extras: () => [/\s/],

  rules: {
    source_file: ($) => repeat(seq($._rules, terminator)),

    _rules: ($) => choice($._expression),

    _expression: ($) =>
      choice(
        $.prefix_expression,
        $.infix_expression,
        $._grouped_expression,
        $.if_expression,

        $.identifier,
        $.integer,
        $.float,
        $.true,
        $.false,
        $.null,
        $.array,
        $.dictionary,
        $.string,
      ),

    prefix_expression: ($) =>
      prec(
        PREC.prefix,
        seq(field("operator", choice("!", "-")), field("right", $._expression)),
      ),

    infix_expression: ($) => {
      const operators = [
        ["-", PREC.sum],
        ["+", PREC.sum],
        ["*", PREC.product],
        ["/", PREC.product],
        ["%", PREC.product],
        ["&", PREC.and],
        ["|", PREC.or],
        ["<", PREC.less_greater],
        ["<=", PREC.less_greater],
        [">", PREC.less_greater],
        [">=", PREC.less_greater],
        ["==", PREC.equals],
        ["!=", PREC.equals],
      ];

      return choice(
        ...operators.map(([operator, precedence]) =>
          prec.left(
            precedence,
            seq(
              field("left", $._expression),
              field("operator", operator),
              field("right", $._expression),
            ),
          ),
        ),
      );
    },

    _grouped_expression: ($) => seq("(", $._expression, ")"),

    if_expression: ($) =>
      seq(
        "if",
        "(",
        field("condition", $._expression),
        ")",
        field("consequence", $.block),

        // Optional else ifs
        repeat(
          seq(
            "else",
            "if",
            "(",
            field("condition", $._expression),
            ")",
            field("consequence", $.block),
          ),
        ),
        // optional else
        optional(seq("else", field("alternative", $.block))),
      ),

    block: ($) =>
      choice(
        // emtpy block
        seq("{", "}"),

        // at least one rule
        seq(
          "{",
          repeat(seq($._rules, terminator)),
          seq($._rules, optional(terminator)),
          "}",
        ),
      ),

    array: ($) =>
      choice(
        // emtpy
        seq("[", "]"),

        // at least one element
        seq(
          "[",
          repeat(seq($._expression, ",")),
          seq($._expression, optional(",")),
          "]",
        ),
      ),

    dictionary: ($) =>
      choice(
        // empty
        seq("{", "}"),

        // at least one elemnt
        seq(
          "{",
          repeat(seq($.dictionary_pair, ",")),
          seq($.dictionary_pair, optional(",")),
          "}",
        ),
      ),
    dictionary_pair: ($) =>
      seq(field("key", $._expression), ":", field("value", $._expression)),

    string: ($) =>
      seq(
        '"',
        repeat(choice($._string_basic_content, $.escape_sequence)),
        token.immediate('"'),
      ),

    _string_basic_content: () => token.immediate(prec(1, /[^"\n\\]+/)),
    escape_sequence: () => token.immediate(/\\./),

    identifier: () => /[a-zA-Z][a-zA-Z_\d]*/,
    integer: () => /\d+/,
    float: () => /\d+\.\d+/,
    true: () => "true",
    false: () => "false",
    null: () => "null",
  },
});
