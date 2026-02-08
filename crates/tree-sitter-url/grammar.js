/**
 * @file Url grammar for tree-sitter
 * @author Andreas <z@zanmato.se>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "url",

  conflicts: $ => [[$.domain_and_port], [$.path]],

  rules: {
    source_file: $ => repeat(seq(
      $.url_line,
      optional("\n")
    )),

    url_line: $ => seq(
      optional($.protocol),
      optional($.domain_and_port),
      $.url_components
    ),

    url_components: $ => seq(
      $.path,
      optional($.query_string)
    ),

    protocol: $ => seq(
      choice("http", "https", "ftp", "ws", "wss"),
      "://"
    ),

    domain_and_port: $ => seq(
      $.domain,
      optional(seq(":", $.port))
    ),

    domain: $ => choice(
      $.hostname,
      /[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)+/
    ),

    hostname: $ => choice(
      "localhost",
      /[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}/
    ),

    port: $ => /\d{1,5}/,

    path: $ => seq(
      repeat1(choice(
        $.variable,
        $.path_param,
        $.path_segment,
        "/"
      ))
    ),

    path_segment: $ => /[^\/\?{:=}&\s][^\/\?{:=}&\s]*/,

    path_param: $ => seq(
      ":",
      /[a-zA-Z_][a-zA-Z0-9_-]*/
    ),

    variable: $ => seq(
      $.variable_delim_start,
      $.variable_name,
      $.variable_delim_end
    ),

    variable_delim_start: $ => "{{",

    variable_delim_end: $ => "}}",

    variable_name: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,

    query_string: $ => seq(
      "?",
      $.query_param,
      repeat(seq(
        "&",
        $.query_param
      ))
    ),

    query_param: $ => seq(
      $.key,
      optional(seq("=", $.value))
    ),

    key: $ => choice(
      $.variable,
      /[a-zA-Z0-9_-]+/
    ),

    value: $ => choice(
      $.variable,
      /[^&]+/
    )
  }
});
