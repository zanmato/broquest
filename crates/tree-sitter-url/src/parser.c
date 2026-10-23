#include "tree_sitter/parser.h"

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#ifdef _MSC_VER
#pragma optimize("", off)
#elif defined(__clang__)
#pragma clang optimize off
#elif defined(__GNUC__)
#pragma GCC optimize ("O0")
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 44
#define LARGE_STATE_COUNT 4
#define SYMBOL_COUNT 42
#define ALIAS_COUNT 0
#define TOKEN_COUNT 24
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 0
#define MAX_ALIAS_SEQUENCE_LENGTH 3
#define PRODUCTION_ID_COUNT 1

enum ts_symbol_identifiers {
  anon_sym_LF = 1,
  anon_sym_http = 2,
  anon_sym_https = 3,
  anon_sym_ftp = 4,
  anon_sym_ws = 5,
  anon_sym_wss = 6,
  anon_sym_COLON_SLASH_SLASH = 7,
  anon_sym_COLON = 8,
  aux_sym_domain_token1 = 9,
  anon_sym_localhost = 10,
  aux_sym_hostname_token1 = 11,
  sym_port = 12,
  anon_sym_SLASH = 13,
  sym_path_segment = 14,
  aux_sym_path_param_token1 = 15,
  sym_variable_delim_start = 16,
  sym_variable_delim_end = 17,
  sym_variable_name = 18,
  anon_sym_QMARK = 19,
  anon_sym_AMP = 20,
  anon_sym_EQ = 21,
  aux_sym_key_token1 = 22,
  aux_sym_value_token1 = 23,
  sym_source_file = 24,
  sym_url_line = 25,
  sym_url_components = 26,
  sym_protocol = 27,
  sym_domain_and_port = 28,
  sym_domain = 29,
  sym_hostname = 30,
  sym_path = 31,
  sym_path_param = 32,
  sym_variable = 33,
  sym_query_string = 34,
  sym_query_param = 35,
  sym_key = 36,
  sym_value = 37,
  aux_sym_source_file_repeat1 = 38,
  aux_sym_path_repeat1 = 39,
  aux_sym_query_string_repeat1 = 40,
  aux_sym_value_repeat1 = 41,
};

static const char * const ts_symbol_names[] = {
  [ts_builtin_sym_end] = "end",
  [anon_sym_LF] = "\n",
  [anon_sym_http] = "http",
  [anon_sym_https] = "https",
  [anon_sym_ftp] = "ftp",
  [anon_sym_ws] = "ws",
  [anon_sym_wss] = "wss",
  [anon_sym_COLON_SLASH_SLASH] = "://",
  [anon_sym_COLON] = ":",
  [aux_sym_domain_token1] = "domain_token1",
  [anon_sym_localhost] = "localhost",
  [aux_sym_hostname_token1] = "hostname_token1",
  [sym_port] = "port",
  [anon_sym_SLASH] = "/",
  [sym_path_segment] = "path_segment",
  [aux_sym_path_param_token1] = "path_param_token1",
  [sym_variable_delim_start] = "variable_delim_start",
  [sym_variable_delim_end] = "variable_delim_end",
  [sym_variable_name] = "variable_name",
  [anon_sym_QMARK] = "\?",
  [anon_sym_AMP] = "&",
  [anon_sym_EQ] = "=",
  [aux_sym_key_token1] = "key_token1",
  [aux_sym_value_token1] = "value_token1",
  [sym_source_file] = "source_file",
  [sym_url_line] = "url_line",
  [sym_url_components] = "url_components",
  [sym_protocol] = "protocol",
  [sym_domain_and_port] = "domain_and_port",
  [sym_domain] = "domain",
  [sym_hostname] = "hostname",
  [sym_path] = "path",
  [sym_path_param] = "path_param",
  [sym_variable] = "variable",
  [sym_query_string] = "query_string",
  [sym_query_param] = "query_param",
  [sym_key] = "key",
  [sym_value] = "value",
  [aux_sym_source_file_repeat1] = "source_file_repeat1",
  [aux_sym_path_repeat1] = "path_repeat1",
  [aux_sym_query_string_repeat1] = "query_string_repeat1",
  [aux_sym_value_repeat1] = "value_repeat1",
};

static const TSSymbol ts_symbol_map[] = {
  [ts_builtin_sym_end] = ts_builtin_sym_end,
  [anon_sym_LF] = anon_sym_LF,
  [anon_sym_http] = anon_sym_http,
  [anon_sym_https] = anon_sym_https,
  [anon_sym_ftp] = anon_sym_ftp,
  [anon_sym_ws] = anon_sym_ws,
  [anon_sym_wss] = anon_sym_wss,
  [anon_sym_COLON_SLASH_SLASH] = anon_sym_COLON_SLASH_SLASH,
  [anon_sym_COLON] = anon_sym_COLON,
  [aux_sym_domain_token1] = aux_sym_domain_token1,
  [anon_sym_localhost] = anon_sym_localhost,
  [aux_sym_hostname_token1] = aux_sym_hostname_token1,
  [sym_port] = sym_port,
  [anon_sym_SLASH] = anon_sym_SLASH,
  [sym_path_segment] = sym_path_segment,
  [aux_sym_path_param_token1] = aux_sym_path_param_token1,
  [sym_variable_delim_start] = sym_variable_delim_start,
  [sym_variable_delim_end] = sym_variable_delim_end,
  [sym_variable_name] = sym_variable_name,
  [anon_sym_QMARK] = anon_sym_QMARK,
  [anon_sym_AMP] = anon_sym_AMP,
  [anon_sym_EQ] = anon_sym_EQ,
  [aux_sym_key_token1] = aux_sym_key_token1,
  [aux_sym_value_token1] = aux_sym_value_token1,
  [sym_source_file] = sym_source_file,
  [sym_url_line] = sym_url_line,
  [sym_url_components] = sym_url_components,
  [sym_protocol] = sym_protocol,
  [sym_domain_and_port] = sym_domain_and_port,
  [sym_domain] = sym_domain,
  [sym_hostname] = sym_hostname,
  [sym_path] = sym_path,
  [sym_path_param] = sym_path_param,
  [sym_variable] = sym_variable,
  [sym_query_string] = sym_query_string,
  [sym_query_param] = sym_query_param,
  [sym_key] = sym_key,
  [sym_value] = sym_value,
  [aux_sym_source_file_repeat1] = aux_sym_source_file_repeat1,
  [aux_sym_path_repeat1] = aux_sym_path_repeat1,
  [aux_sym_query_string_repeat1] = aux_sym_query_string_repeat1,
  [aux_sym_value_repeat1] = aux_sym_value_repeat1,
};

static const TSSymbolMetadata ts_symbol_metadata[] = {
  [ts_builtin_sym_end] = {
    .visible = false,
    .named = true,
  },
  [anon_sym_LF] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_http] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_https] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_ftp] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_ws] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_wss] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COLON_SLASH_SLASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COLON] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_domain_token1] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_localhost] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_hostname_token1] = {
    .visible = false,
    .named = false,
  },
  [sym_port] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_SLASH] = {
    .visible = true,
    .named = false,
  },
  [sym_path_segment] = {
    .visible = true,
    .named = true,
  },
  [aux_sym_path_param_token1] = {
    .visible = false,
    .named = false,
  },
  [sym_variable_delim_start] = {
    .visible = true,
    .named = true,
  },
  [sym_variable_delim_end] = {
    .visible = true,
    .named = true,
  },
  [sym_variable_name] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_QMARK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_AMP] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_EQ] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_key_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_value_token1] = {
    .visible = false,
    .named = false,
  },
  [sym_source_file] = {
    .visible = true,
    .named = true,
  },
  [sym_url_line] = {
    .visible = true,
    .named = true,
  },
  [sym_url_components] = {
    .visible = true,
    .named = true,
  },
  [sym_protocol] = {
    .visible = true,
    .named = true,
  },
  [sym_domain_and_port] = {
    .visible = true,
    .named = true,
  },
  [sym_domain] = {
    .visible = true,
    .named = true,
  },
  [sym_hostname] = {
    .visible = true,
    .named = true,
  },
  [sym_path] = {
    .visible = true,
    .named = true,
  },
  [sym_path_param] = {
    .visible = true,
    .named = true,
  },
  [sym_variable] = {
    .visible = true,
    .named = true,
  },
  [sym_query_string] = {
    .visible = true,
    .named = true,
  },
  [sym_query_param] = {
    .visible = true,
    .named = true,
  },
  [sym_key] = {
    .visible = true,
    .named = true,
  },
  [sym_value] = {
    .visible = true,
    .named = true,
  },
  [aux_sym_source_file_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_path_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_query_string_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_value_repeat1] = {
    .visible = false,
    .named = false,
  },
};

static const TSSymbol ts_alias_sequences[PRODUCTION_ID_COUNT][MAX_ALIAS_SEQUENCE_LENGTH] = {
  [0] = {0},
};

static const uint16_t ts_non_terminal_alias_map[] = {
  0,
};

static const TSStateId ts_primary_state_ids[STATE_COUNT] = {
  [0] = 0,
  [1] = 1,
  [2] = 2,
  [3] = 3,
  [4] = 4,
  [5] = 5,
  [6] = 6,
  [7] = 7,
  [8] = 8,
  [9] = 9,
  [10] = 10,
  [11] = 11,
  [12] = 12,
  [13] = 13,
  [14] = 14,
  [15] = 8,
  [16] = 16,
  [17] = 17,
  [18] = 18,
  [19] = 19,
  [20] = 20,
  [21] = 21,
  [22] = 22,
  [23] = 23,
  [24] = 24,
  [25] = 25,
  [26] = 26,
  [27] = 27,
  [28] = 28,
  [29] = 29,
  [30] = 30,
  [31] = 31,
  [32] = 32,
  [33] = 33,
  [34] = 34,
  [35] = 35,
  [36] = 36,
  [37] = 37,
  [38] = 38,
  [39] = 39,
  [40] = 40,
  [41] = 41,
  [42] = 39,
  [43] = 38,
};

static TSCharacterRange sym_path_segment_character_set_1[] = {
  {0, 0x08}, {0x0e, 0x1f}, {'!', '%'}, {'\'', '.'}, {'0', '9'}, {';', '<'}, {'>', '>'}, {'@', 'z'},
  {'|', '|'}, {'~', 0x10ffff},
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(28);
      ADVANCE_MAP(
        '&', 622,
        '/', 200,
        ':', 48,
        '=', 623,
        '?', 621,
        'f', 16,
        'h', 19,
        'l', 10,
        'w', 14,
        '{', 21,
        '}', 22,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(0);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(199);
      END_STATE();
    case 1:
      if (lookahead == '/') ADVANCE(200);
      if (lookahead == ':') ADVANCE(47);
      if (lookahead == 'l') ADVANCE(202);
      if (lookahead == '{') ADVANCE(21);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(1);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(207);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(206);
      if (lookahead != 0 &&
          lookahead != '&' &&
          lookahead != '=' &&
          lookahead != '?' &&
          lookahead != '}') ADVANCE(615);
      END_STATE();
    case 2:
      if (lookahead == '/') ADVANCE(200);
      if (lookahead == ':') ADVANCE(47);
      if (lookahead == '{') ADVANCE(21);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(2);
      if (lookahead != 0 &&
          lookahead != '&' &&
          lookahead != '=' &&
          lookahead != '?' &&
          lookahead != '}') ADVANCE(615);
      END_STATE();
    case 3:
      if (lookahead == '/') ADVANCE(4);
      END_STATE();
    case 4:
      if (lookahead == '/') ADVANCE(46);
      END_STATE();
    case 5:
      if (lookahead == ':') ADVANCE(3);
      if (lookahead == '{') ADVANCE(21);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(5);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(624);
      END_STATE();
    case 6:
      if (lookahead == 'a') ADVANCE(9);
      END_STATE();
    case 7:
      if (lookahead == 'c') ADVANCE(6);
      END_STATE();
    case 8:
      if (lookahead == 'h') ADVANCE(11);
      END_STATE();
    case 9:
      if (lookahead == 'l') ADVANCE(8);
      END_STATE();
    case 10:
      if (lookahead == 'o') ADVANCE(7);
      END_STATE();
    case 11:
      if (lookahead == 'o') ADVANCE(15);
      END_STATE();
    case 12:
      if (lookahead == 'p') ADVANCE(37);
      END_STATE();
    case 13:
      if (lookahead == 'p') ADVANCE(33);
      END_STATE();
    case 14:
      if (lookahead == 's') ADVANCE(42);
      END_STATE();
    case 15:
      if (lookahead == 's') ADVANCE(17);
      END_STATE();
    case 16:
      if (lookahead == 't') ADVANCE(12);
      END_STATE();
    case 17:
      if (lookahead == 't') ADVANCE(192);
      END_STATE();
    case 18:
      if (lookahead == 't') ADVANCE(13);
      END_STATE();
    case 19:
      if (lookahead == 't') ADVANCE(18);
      END_STATE();
    case 20:
      if (lookahead == '{') ADVANCE(21);
      if (lookahead == '\n' ||
          lookahead == '\r') SKIP(20);
      if (('\t' <= lookahead && lookahead <= '\f') ||
          lookahead == ' ') ADVANCE(626);
      if (lookahead != 0 &&
          lookahead != '&' &&
          lookahead != '}') ADVANCE(627);
      END_STATE();
    case 21:
      if (lookahead == '{') ADVANCE(617);
      END_STATE();
    case 22:
      if (lookahead == '}') ADVANCE(618);
      END_STATE();
    case 23:
      if (lookahead == '\n' ||
          lookahead == '\r') SKIP(23);
      if (('\t' <= lookahead && lookahead <= '\f') ||
          lookahead == ' ') ADVANCE(619);
      if (lookahead != 0 &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(620);
      END_STATE();
    case 24:
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(24);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(616);
      END_STATE();
    case 25:
      if (eof) ADVANCE(28);
      ADVANCE_MAP(
        '\n', 29,
        '&', 622,
        '/', 200,
        ':', 47,
        '=', 623,
        '?', 621,
        'f', 204,
        'h', 205,
        'l', 202,
        'w', 203,
        '{', 21,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(25);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(207);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(206);
      if (lookahead != 0 &&
          lookahead != '}') ADVANCE(615);
      END_STATE();
    case 26:
      if (eof) ADVANCE(28);
      if (lookahead == '\n') ADVANCE(30);
      if (lookahead == '\r') SKIP(26);
      if (lookahead == '&') ADVANCE(622);
      if (lookahead == '/') ADVANCE(201);
      if (lookahead == ':') ADVANCE(49);
      if (lookahead == 'f') ADVANCE(210);
      if (lookahead == 'h') ADVANCE(211);
      if (lookahead == 'l') ADVANCE(208);
      if (lookahead == 'w') ADVANCE(209);
      if (lookahead == '{') ADVANCE(21);
      if (lookahead == '=' ||
          lookahead == '?') ADVANCE(627);
      if (('\t' <= lookahead && lookahead <= '\f') ||
          lookahead == ' ') ADVANCE(625);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(213);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(212);
      if (lookahead != 0 &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 27:
      if (eof) ADVANCE(28);
      if (lookahead == '/') ADVANCE(200);
      if (lookahead == ':') ADVANCE(47);
      if (lookahead == 'f') ADVANCE(204);
      if (lookahead == 'h') ADVANCE(205);
      if (lookahead == 'l') ADVANCE(202);
      if (lookahead == 'w') ADVANCE(203);
      if (lookahead == '{') ADVANCE(21);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(27);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(207);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(206);
      if (lookahead != 0 &&
          lookahead != '&' &&
          lookahead != '=' &&
          lookahead != '?' &&
          lookahead != '}') ADVANCE(615);
      END_STATE();
    case 28:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 29:
      ACCEPT_TOKEN(anon_sym_LF);
      if (lookahead == '\n') ADVANCE(29);
      END_STATE();
    case 30:
      ACCEPT_TOKEN(anon_sym_LF);
      if (lookahead == '\n') ADVANCE(30);
      if (('\t' <= lookahead && lookahead <= '\f') ||
          lookahead == ' ') ADVANCE(625);
      END_STATE();
    case 31:
      ACCEPT_TOKEN(anon_sym_http);
      if (lookahead == '-') ADVANCE(550);
      if (lookahead == '.') ADVANCE(612);
      if (lookahead == 's') ADVANCE(35);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(549);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 32:
      ACCEPT_TOKEN(anon_sym_http);
      ADVANCE_MAP(
        '-', 553,
        '.', 605,
        's', 36,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(552);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 33:
      ACCEPT_TOKEN(anon_sym_http);
      if (lookahead == 's') ADVANCE(34);
      END_STATE();
    case 34:
      ACCEPT_TOKEN(anon_sym_https);
      END_STATE();
    case 35:
      ACCEPT_TOKEN(anon_sym_https);
      if (lookahead == '-') ADVANCE(558);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(557);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 36:
      ACCEPT_TOKEN(anon_sym_https);
      ADVANCE_MAP(
        '-', 561,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(560);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 37:
      ACCEPT_TOKEN(anon_sym_ftp);
      END_STATE();
    case 38:
      ACCEPT_TOKEN(anon_sym_ftp);
      if (lookahead == '-') ADVANCE(578);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(577);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 39:
      ACCEPT_TOKEN(anon_sym_ftp);
      ADVANCE_MAP(
        '-', 581,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(580);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 40:
      ACCEPT_TOKEN(anon_sym_ws);
      if (lookahead == '-') ADVANCE(568);
      if (lookahead == '.') ADVANCE(612);
      if (lookahead == 's') ADVANCE(44);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(566);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 41:
      ACCEPT_TOKEN(anon_sym_ws);
      ADVANCE_MAP(
        '-', 573,
        '.', 605,
        's', 45,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(571);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 42:
      ACCEPT_TOKEN(anon_sym_ws);
      if (lookahead == 's') ADVANCE(43);
      END_STATE();
    case 43:
      ACCEPT_TOKEN(anon_sym_wss);
      END_STATE();
    case 44:
      ACCEPT_TOKEN(anon_sym_wss);
      if (lookahead == '-') ADVANCE(578);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(577);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 45:
      ACCEPT_TOKEN(anon_sym_wss);
      ADVANCE_MAP(
        '-', 581,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(580);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 46:
      ACCEPT_TOKEN(anon_sym_COLON_SLASH_SLASH);
      END_STATE();
    case 47:
      ACCEPT_TOKEN(anon_sym_COLON);
      END_STATE();
    case 48:
      ACCEPT_TOKEN(anon_sym_COLON);
      if (lookahead == '/') ADVANCE(4);
      END_STATE();
    case 49:
      ACCEPT_TOKEN(anon_sym_COLON);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r' &&
          lookahead != '&' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(627);
      END_STATE();
    case 50:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(596);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(180);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(181);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 51:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(596);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(181);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 52:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(596);
      if (lookahead == '.') ADVANCE(610);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(182);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(181);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 53:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(596);
      if (lookahead == '.') ADVANCE(611);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(183);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(181);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 54:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 597,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(184);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(185);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 55:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 597,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(185);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 56:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 597,
        '.', 603,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(186);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(185);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 57:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 597,
        '.', 604,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(187);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(185);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 58:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(614);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(188);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 59:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 607,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(190);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 60:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(230);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(62);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 61:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 231,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(63);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 62:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(220);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(58);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 63:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 223,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(59);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 64:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(242);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(66);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 65:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 243,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(67);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 66:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(228);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(60);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 67:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 229,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(61);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 68:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(254);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(70);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 69:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 255,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(71);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 70:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(236);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 71:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 237,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(65);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 72:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(266);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(74);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 73:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 267,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(75);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 74:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(248);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(68);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 75:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 249,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(69);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 76:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(278);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(78);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 77:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 279,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(79);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 78:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(260);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(72);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 79:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 261,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(73);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 80:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(290);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(82);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 81:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 291,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(83);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 82:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(272);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(76);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 83:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 273,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(77);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 84:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(302);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(86);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 85:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 303,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(87);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 86:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(284);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(80);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 87:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 285,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(81);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 88:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(314);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(90);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 89:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 315,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(91);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 90:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(296);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(84);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 91:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 297,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 92:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(326);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(94);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 93:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 327,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(95);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 94:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(308);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(88);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 95:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 309,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(89);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 96:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(338);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(98);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 97:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 339,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(99);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 98:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(320);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(92);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 99:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 321,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(93);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 100:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(350);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(102);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 101:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 351,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(103);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 102:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(332);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(96);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 103:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 333,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(97);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 104:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(362);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(106);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 105:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 363,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(107);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 106:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(344);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(100);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 107:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 345,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(101);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 108:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(374);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(110);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 109:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 375,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(111);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 110:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(356);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(104);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 111:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 357,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(105);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 112:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(386);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(114);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 113:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 387,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(115);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 114:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(368);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(108);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 115:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 369,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(109);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 116:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(398);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(118);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 117:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 399,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(119);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 118:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(380);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(112);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 119:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 381,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(113);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 120:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(410);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(122);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 121:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 411,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(123);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 122:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(392);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(116);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 123:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 393,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(117);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 124:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(422);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(126);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 125:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 423,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(127);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 126:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(404);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(120);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 127:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 405,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(121);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 128:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(434);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(130);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 129:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 435,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(131);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 130:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(416);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(124);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 131:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 417,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(125);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 132:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(446);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(134);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 133:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 447,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(135);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 134:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(428);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(128);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 135:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 429,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(129);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 136:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(458);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(138);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 137:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 459,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(139);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 138:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(440);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(132);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 139:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 441,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(133);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 140:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(470);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(142);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 141:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 471,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(143);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 142:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(452);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(136);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 143:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 453,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(137);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 144:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(482);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(146);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 145:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 483,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(147);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 146:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(464);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(140);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 147:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 465,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(141);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 148:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(494);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(150);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 149:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 495,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(151);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 150:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(476);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(144);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 151:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 477,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(145);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 152:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(506);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(154);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 153:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 507,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(155);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 154:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(488);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(148);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 155:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 489,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(149);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 156:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(518);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(158);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 157:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 519,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(159);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 158:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(500);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(152);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 159:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 501,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(153);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 160:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(530);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(162);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 161:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 531,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 162:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(512);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(156);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 163:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 513,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(157);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 164:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(546);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(166);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 165:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 547,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(167);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 166:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(524);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(160);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 167:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 525,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(161);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 168:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(562);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(170);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 169:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 563,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(171);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 170:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(538);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(164);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 171:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 539,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(165);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 172:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(582);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(178);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 173:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(582);
      if (lookahead == '.') ADVANCE(610);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(178);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 174:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(582);
      if (lookahead == '.') ADVANCE(611);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(178);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 175:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 583,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(179);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 176:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 583,
        '.', 603,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(179);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 177:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 583,
        '.', 604,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(179);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 178:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(554);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(168);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 179:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 555,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(169);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 180:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(574);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(172);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(172);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 181:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(574);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(172);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 182:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(574);
      if (lookahead == '.') ADVANCE(610);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(173);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(172);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 183:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(574);
      if (lookahead == '.') ADVANCE(611);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(174);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(172);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 184:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 575,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(175);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(175);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 185:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 575,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(175);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 186:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 575,
        '.', 603,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(176);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(175);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 187:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '-', 575,
        '.', 604,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(177);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(175);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 188:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(189);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 189:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '.') ADVANCE(612);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 190:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(191);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 191:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      ADVANCE_MAP(
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 192:
      ACCEPT_TOKEN(anon_sym_localhost);
      END_STATE();
    case 193:
      ACCEPT_TOKEN(anon_sym_localhost);
      if (lookahead == '-') ADVANCE(527);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(526);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 194:
      ACCEPT_TOKEN(anon_sym_localhost);
      ADVANCE_MAP(
        '-', 529,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(528);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 195:
      ACCEPT_TOKEN(sym_port);
      END_STATE();
    case 196:
      ACCEPT_TOKEN(sym_port);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(195);
      END_STATE();
    case 197:
      ACCEPT_TOKEN(sym_port);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(196);
      END_STATE();
    case 198:
      ACCEPT_TOKEN(sym_port);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(197);
      END_STATE();
    case 199:
      ACCEPT_TOKEN(sym_port);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(198);
      END_STATE();
    case 200:
      ACCEPT_TOKEN(anon_sym_SLASH);
      END_STATE();
    case 201:
      ACCEPT_TOKEN(anon_sym_SLASH);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r' &&
          lookahead != '&' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(627);
      END_STATE();
    case 202:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(589);
      if (lookahead == '.') ADVANCE(612);
      if (lookahead == 'o') ADVANCE(584);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(587);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 203:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(589);
      if (lookahead == '.') ADVANCE(612);
      if (lookahead == 's') ADVANCE(40);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(587);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 204:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(589);
      if (lookahead == '.') ADVANCE(612);
      if (lookahead == 't') ADVANCE(585);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(587);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 205:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(589);
      if (lookahead == '.') ADVANCE(612);
      if (lookahead == 't') ADVANCE(586);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(587);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 206:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(589);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(587);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 207:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(589);
      if (lookahead == '.') ADVANCE(609);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(588);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(587);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 208:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 595,
        '.', 605,
        'o', 590,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(593);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 209:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 595,
        '.', 605,
        's', 41,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(593);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 210:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 595,
        '.', 605,
        't', 591,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(593);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 211:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 595,
        '.', 605,
        't', 592,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(593);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 212:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 595,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(593);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 213:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 595,
        '.', 602,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(594);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(593);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 214:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(613);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(598);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 215:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(613);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(598);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 216:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 606,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(600);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 217:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 606,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(600);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 218:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(225);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(224);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 219:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(225);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(224);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 220:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(614);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(188);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 221:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 227,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(226);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 222:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 227,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(226);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 223:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 607,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(190);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 224:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(215);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(214);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 225:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(215);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(214);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 226:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 217,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(216);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 227:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 217,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(216);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 228:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(230);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(62);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 229:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 231,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(63);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 230:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(220);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(58);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 231:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 223,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(59);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 232:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(239);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(238);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 233:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(239);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(238);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 234:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 241,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(240);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 235:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 241,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(240);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 236:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(242);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(66);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 237:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 243,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(67);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 238:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(219);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(218);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 239:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(219);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(218);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 240:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 222,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(221);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 241:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 222,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(221);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 242:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(228);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(60);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 243:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 229,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(61);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 244:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(251);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(250);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 245:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(251);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(250);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 246:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 253,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(252);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 247:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 253,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(252);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 248:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(254);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(70);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 249:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 255,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(71);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 250:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(233);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(232);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 251:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(233);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(232);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 252:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 235,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(234);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 253:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 235,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(234);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 254:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(236);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 255:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 237,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(65);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 256:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(263);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(262);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 257:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(263);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(262);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 258:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 265,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(264);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 259:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 265,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(264);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 260:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(266);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(74);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 261:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 267,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(75);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 262:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(245);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(244);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 263:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(245);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(244);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 264:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 247,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(246);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 265:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 247,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(246);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 266:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(248);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(68);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 267:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 249,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(69);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 268:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(275);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(274);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 269:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(275);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(274);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 270:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 277,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(276);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 271:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 277,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(276);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 272:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(278);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(78);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 273:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 279,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(79);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 274:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(257);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(256);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 275:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(257);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(256);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 276:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 259,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(258);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 277:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 259,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(258);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 278:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(260);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(72);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 279:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 261,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(73);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 280:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(287);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(286);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 281:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(287);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(286);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 282:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 289,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(288);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 283:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 289,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(288);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 284:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(290);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(82);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 285:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 291,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(83);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 286:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(269);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(268);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 287:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(269);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(268);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 288:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 271,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(270);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 289:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 271,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(270);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 290:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(272);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(76);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 291:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 273,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(77);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 292:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(299);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(298);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 293:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(299);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(298);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 294:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 301,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(300);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 295:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 301,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(300);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 296:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(302);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(86);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 297:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 303,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(87);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 298:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(281);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(280);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 299:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(281);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(280);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 300:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 283,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(282);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 301:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 283,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(282);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 302:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(284);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(80);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 303:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 285,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(81);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 304:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(311);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(310);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 305:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(311);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(310);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 306:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 313,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(312);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 307:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 313,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(312);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 308:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(314);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(90);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 309:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 315,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(91);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 310:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(293);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(292);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 311:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(293);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(292);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 312:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 295,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(294);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 313:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 295,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(294);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 314:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(296);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(84);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 315:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 297,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 316:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(323);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(322);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 317:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(323);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(322);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 318:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 325,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(324);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 319:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 325,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(324);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 320:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(326);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(94);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 321:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 327,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(95);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 322:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(305);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(304);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 323:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(305);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(304);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 324:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 307,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(306);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 325:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 307,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(306);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 326:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(308);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(88);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 327:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 309,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(89);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 328:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(335);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(334);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 329:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(335);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(334);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 330:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 337,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(336);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 331:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 337,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(336);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 332:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(338);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(98);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 333:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 339,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(99);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 334:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(317);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(316);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 335:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(317);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(316);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 336:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 319,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(318);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 337:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 319,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(318);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 338:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(320);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(92);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 339:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 321,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(93);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 340:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(347);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(346);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 341:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(347);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(346);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 342:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 349,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(348);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 343:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 349,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(348);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 344:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(350);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(102);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 345:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 351,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(103);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 346:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(329);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 347:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(329);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 348:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 331,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(330);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 349:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 331,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(330);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 350:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(332);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(96);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 351:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 333,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(97);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 352:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(359);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(358);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 353:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(359);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(358);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 354:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 361,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(360);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 355:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 361,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(360);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 356:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(362);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(106);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 357:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 363,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(107);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 358:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(341);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(340);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 359:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(341);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(340);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 360:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 343,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(342);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 361:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 343,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(342);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 362:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(344);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(100);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 363:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 345,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(101);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 364:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(371);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(370);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 365:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(371);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(370);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 366:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 373,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(372);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 367:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 373,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(372);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 368:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(374);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(110);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 369:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 375,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(111);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 370:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(353);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(352);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 371:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(353);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(352);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 372:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 355,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(354);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 373:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 355,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(354);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 374:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(356);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(104);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 375:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 357,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(105);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 376:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(383);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(382);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 377:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(383);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(382);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 378:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 385,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(384);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 379:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 385,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(384);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 380:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(386);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(114);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 381:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 387,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(115);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 382:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(365);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(364);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 383:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(365);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(364);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 384:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 367,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(366);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 385:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 367,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(366);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 386:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(368);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(108);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 387:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 369,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(109);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 388:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(395);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(394);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 389:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(395);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(394);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 390:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 397,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(396);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 391:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 397,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(396);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 392:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(398);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(118);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 393:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 399,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(119);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 394:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(377);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(376);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 395:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(377);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(376);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 396:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 379,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(378);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 397:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 379,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(378);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 398:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(380);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(112);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 399:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 381,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(113);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 400:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(407);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(406);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 401:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(407);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(406);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 402:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 409,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(408);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 403:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 409,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(408);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 404:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(410);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(122);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 405:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 411,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(123);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 406:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(389);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(388);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 407:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(389);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(388);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 408:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 391,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(390);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 409:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 391,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(390);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 410:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(392);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(116);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 411:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 393,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(117);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 412:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(419);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(418);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 413:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(419);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(418);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 414:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 421,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(420);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 415:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 421,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(420);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 416:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(422);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(126);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 417:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 423,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(127);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 418:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(401);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(400);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 419:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(401);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(400);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 420:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 403,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(402);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 421:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 403,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(402);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 422:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(404);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(120);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 423:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 405,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(121);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 424:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(431);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(430);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 425:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(431);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(430);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 426:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 433,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(432);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 427:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 433,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(432);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 428:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(434);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(130);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 429:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 435,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(131);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 430:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(413);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(412);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 431:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(413);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(412);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 432:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 415,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(414);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 433:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 415,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(414);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 434:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(416);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(124);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 435:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 417,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(125);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 436:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(443);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(442);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 437:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(443);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(442);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 438:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 445,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(444);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 439:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 445,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(444);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 440:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(446);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(134);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 441:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 447,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(135);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 442:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(425);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(424);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 443:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(425);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(424);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 444:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 427,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(426);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 445:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 427,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(426);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 446:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(428);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(128);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 447:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 429,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(129);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 448:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(455);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(454);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 449:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(455);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(454);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 450:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 457,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(456);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 451:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 457,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(456);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 452:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(458);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(138);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 453:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 459,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(139);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 454:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(437);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(436);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 455:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(437);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(436);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 456:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 439,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(438);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 457:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 439,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(438);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 458:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(440);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(132);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 459:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 441,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(133);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 460:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(467);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(466);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 461:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(467);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(466);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 462:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 469,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(468);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 463:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 469,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(468);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 464:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(470);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(142);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 465:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 471,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(143);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 466:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(449);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(448);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 467:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(449);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(448);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 468:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 451,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(450);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 469:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 451,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(450);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 470:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(452);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(136);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 471:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 453,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(137);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 472:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(479);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(478);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 473:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(479);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(478);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 474:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 481,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(480);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 475:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 481,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(480);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 476:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(482);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(146);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 477:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 483,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(147);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 478:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(461);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(460);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 479:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(461);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(460);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 480:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 463,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(462);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 481:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 463,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(462);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 482:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(464);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(140);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 483:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 465,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(141);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 484:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(491);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(490);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 485:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(491);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(490);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 486:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 493,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(492);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 487:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 493,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(492);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 488:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(494);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(150);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 489:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 495,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(151);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 490:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(473);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(472);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 491:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(473);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(472);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 492:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 475,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(474);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 493:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 475,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(474);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 494:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(476);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(144);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 495:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 477,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(145);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 496:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(503);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(502);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 497:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(503);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(502);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 498:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 505,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(504);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 499:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 505,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(504);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 500:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(506);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(154);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 501:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 507,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(155);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 502:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(485);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(484);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 503:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(485);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(484);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 504:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 487,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(486);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 505:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 487,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(486);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 506:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(488);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(148);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 507:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 489,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(149);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 508:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(515);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(514);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 509:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(515);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(514);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 510:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 517,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(516);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 511:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 517,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(516);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 512:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(518);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(158);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 513:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 519,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(159);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 514:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(497);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(496);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 515:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(497);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(496);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 516:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 499,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(498);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 517:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 499,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(498);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 518:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(500);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(152);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 519:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 501,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(153);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 520:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(527);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(526);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 521:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(527);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(526);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 522:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 529,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(528);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 523:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 529,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(528);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 524:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(530);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(162);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 525:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 531,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 526:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(509);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(508);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 527:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(509);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(508);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 528:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 511,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(510);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 529:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 511,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(510);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 530:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(512);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(156);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 531:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 513,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(157);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 532:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(542);
      if (lookahead == '.') ADVANCE(612);
      if (lookahead == 's') ADVANCE(540);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(541);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 533:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(542);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(541);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 534:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(542);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(541);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 535:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 545,
        '.', 605,
        's', 543,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(544);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 536:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 545,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(544);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 537:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 545,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(544);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 538:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(546);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(166);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 539:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 547,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(167);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 540:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(521);
      if (lookahead == '.') ADVANCE(612);
      if (lookahead == 't') ADVANCE(193);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(520);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 541:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(521);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(520);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 542:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(521);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(520);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 543:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 523,
        '.', 605,
        't', 194,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(522);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 544:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 523,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(522);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 545:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 523,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(522);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 546:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(524);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(160);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 547:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 525,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(161);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 548:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(558);
      if (lookahead == '.') ADVANCE(612);
      if (lookahead == 'h') ADVANCE(556);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(557);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 549:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(558);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(557);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 550:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(558);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(557);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 551:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 561,
        '.', 605,
        'h', 559,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(560);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 552:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 561,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(560);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 553:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 561,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(560);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 554:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(562);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(170);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 555:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 563,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(171);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 556:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(534);
      if (lookahead == '.') ADVANCE(612);
      if (lookahead == 'o') ADVANCE(532);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(533);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 557:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(534);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(533);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 558:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(534);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(533);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 559:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 537,
        '.', 605,
        'o', 535,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(536);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 560:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 537,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(536);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 561:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 537,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(536);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 562:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(538);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(164);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 563:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 539,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(165);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 564:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(578);
      if (lookahead == '.') ADVANCE(612);
      if (lookahead == 'a') ADVANCE(576);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(577);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 565:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(578);
      if (lookahead == '.') ADVANCE(612);
      if (lookahead == 'p') ADVANCE(31);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(577);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 566:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(578);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(577);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 567:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(578);
      if (lookahead == '.') ADVANCE(609);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(577);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 568:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(578);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(577);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 569:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 581,
        '.', 605,
        'a', 579,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(580);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 570:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 581,
        '.', 605,
        'p', 32,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(580);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 571:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 581,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(580);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 572:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 581,
        '.', 602,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(580);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 573:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 581,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(580);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 574:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(582);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(178);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 575:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 583,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(179);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 576:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(550);
      if (lookahead == '.') ADVANCE(612);
      if (lookahead == 'l') ADVANCE(548);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(549);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 577:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(550);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(549);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 578:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(550);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(549);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 579:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 553,
        '.', 605,
        'l', 551,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(552);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 580:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 553,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(552);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 581:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 553,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(552);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 582:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(554);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(168);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 583:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 555,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(169);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 584:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(568);
      if (lookahead == '.') ADVANCE(612);
      if (lookahead == 'c') ADVANCE(564);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(566);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 585:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(568);
      if (lookahead == '.') ADVANCE(612);
      if (lookahead == 'p') ADVANCE(38);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(566);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 586:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(568);
      if (lookahead == '.') ADVANCE(612);
      if (lookahead == 't') ADVANCE(565);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(566);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 587:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(568);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(566);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 588:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(568);
      if (lookahead == '.') ADVANCE(609);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(567);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(566);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 589:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(568);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(566);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 590:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 573,
        '.', 605,
        'c', 569,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(571);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 591:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 573,
        '.', 605,
        'p', 39,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(571);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 592:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 573,
        '.', 605,
        't', 570,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(571);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 593:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 573,
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(571);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 594:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 573,
        '.', 602,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(572);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(571);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 595:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 573,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(571);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 596:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(574);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(172);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 597:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '-', 575,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(175);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 598:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '.') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(599);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 599:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '.') ADVANCE(612);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 600:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(601);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 601:
      ACCEPT_TOKEN(sym_path_segment);
      ADVANCE_MAP(
        '.', 605,
        '\t', 627,
        0x0b, 627,
        '\f', 627,
        ' ', 627,
        '/', 627,
        ':', 627,
        '=', 627,
        '?', 627,
      );
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 602:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ' ||
          lookahead == '/' ||
          lookahead == ':' ||
          lookahead == '=' ||
          lookahead == '?') ADVANCE(627);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(57);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(55);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 603:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ' ||
          lookahead == '/' ||
          lookahead == ':' ||
          lookahead == '=' ||
          lookahead == '?') ADVANCE(627);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(54);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(55);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 604:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ' ||
          lookahead == '/' ||
          lookahead == ':' ||
          lookahead == '=' ||
          lookahead == '?') ADVANCE(627);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(56);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(55);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 605:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ' ||
          lookahead == '/' ||
          lookahead == ':' ||
          lookahead == '=' ||
          lookahead == '?') ADVANCE(627);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(55);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 606:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ' ||
          lookahead == '/' ||
          lookahead == ':' ||
          lookahead == '=' ||
          lookahead == '?') ADVANCE(627);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(601);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 607:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ' ||
          lookahead == '/' ||
          lookahead == ':' ||
          lookahead == '=' ||
          lookahead == '?') ADVANCE(627);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(191);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 608:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ' ||
          lookahead == '/' ||
          lookahead == ':' ||
          lookahead == '=' ||
          lookahead == '?') ADVANCE(627);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 609:
      ACCEPT_TOKEN(sym_path_segment);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(53);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(51);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 610:
      ACCEPT_TOKEN(sym_path_segment);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(50);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(51);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 611:
      ACCEPT_TOKEN(sym_path_segment);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(52);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(51);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 612:
      ACCEPT_TOKEN(sym_path_segment);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(51);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 613:
      ACCEPT_TOKEN(sym_path_segment);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(599);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 614:
      ACCEPT_TOKEN(sym_path_segment);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(189);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 615:
      ACCEPT_TOKEN(sym_path_segment);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(615);
      END_STATE();
    case 616:
      ACCEPT_TOKEN(aux_sym_path_param_token1);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(616);
      END_STATE();
    case 617:
      ACCEPT_TOKEN(sym_variable_delim_start);
      END_STATE();
    case 618:
      ACCEPT_TOKEN(sym_variable_delim_end);
      END_STATE();
    case 619:
      ACCEPT_TOKEN(sym_variable_name);
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ') ADVANCE(619);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(620);
      END_STATE();
    case 620:
      ACCEPT_TOKEN(sym_variable_name);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(620);
      END_STATE();
    case 621:
      ACCEPT_TOKEN(anon_sym_QMARK);
      END_STATE();
    case 622:
      ACCEPT_TOKEN(anon_sym_AMP);
      END_STATE();
    case 623:
      ACCEPT_TOKEN(anon_sym_EQ);
      END_STATE();
    case 624:
      ACCEPT_TOKEN(aux_sym_key_token1);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(624);
      END_STATE();
    case 625:
      ACCEPT_TOKEN(aux_sym_value_token1);
      ADVANCE_MAP(
        '\n', 30,
        '/', 201,
        ':', 49,
        'f', 210,
        'h', 211,
        'l', 208,
        'w', 209,
        '=', 627,
        '?', 627,
        '\t', 625,
        0x0b, 625,
        '\f', 625,
        ' ', 625,
      );
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(213);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(212);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          (lookahead < 'a' || '{' < lookahead) &&
          lookahead != '}') ADVANCE(608);
      END_STATE();
    case 626:
      ACCEPT_TOKEN(aux_sym_value_token1);
      if (lookahead == '\t' ||
          lookahead == 0x0b ||
          lookahead == '\f' ||
          lookahead == ' ') ADVANCE(626);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead) &&
          lookahead != '&' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(627);
      END_STATE();
    case 627:
      ACCEPT_TOKEN(aux_sym_value_token1);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '\r' &&
          lookahead != '&' &&
          lookahead != '{' &&
          lookahead != '}') ADVANCE(627);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 27},
  [2] = {.lex_state = 27},
  [3] = {.lex_state = 27},
  [4] = {.lex_state = 25},
  [5] = {.lex_state = 25},
  [6] = {.lex_state = 26},
  [7] = {.lex_state = 26},
  [8] = {.lex_state = 25},
  [9] = {.lex_state = 25},
  [10] = {.lex_state = 25},
  [11] = {.lex_state = 25},
  [12] = {.lex_state = 25},
  [13] = {.lex_state = 25},
  [14] = {.lex_state = 25},
  [15] = {.lex_state = 26},
  [16] = {.lex_state = 25},
  [17] = {.lex_state = 1},
  [18] = {.lex_state = 25},
  [19] = {.lex_state = 25},
  [20] = {.lex_state = 25},
  [21] = {.lex_state = 25},
  [22] = {.lex_state = 25},
  [23] = {.lex_state = 25},
  [24] = {.lex_state = 25},
  [25] = {.lex_state = 27},
  [26] = {.lex_state = 2},
  [27] = {.lex_state = 2},
  [28] = {.lex_state = 1},
  [29] = {.lex_state = 5},
  [30] = {.lex_state = 20},
  [31] = {.lex_state = 5},
  [32] = {.lex_state = 2},
  [33] = {.lex_state = 2},
  [34] = {.lex_state = 2},
  [35] = {.lex_state = 2},
  [36] = {.lex_state = 5},
  [37] = {.lex_state = 0},
  [38] = {.lex_state = 23},
  [39] = {.lex_state = 0},
  [40] = {.lex_state = 24},
  [41] = {.lex_state = 0},
  [42] = {.lex_state = 0},
  [43] = {.lex_state = 23},
};

static const uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {
  [0] = {
    [ts_builtin_sym_end] = ACTIONS(1),
    [anon_sym_http] = ACTIONS(1),
    [anon_sym_https] = ACTIONS(1),
    [anon_sym_ftp] = ACTIONS(1),
    [anon_sym_ws] = ACTIONS(1),
    [anon_sym_wss] = ACTIONS(1),
    [anon_sym_COLON_SLASH_SLASH] = ACTIONS(1),
    [anon_sym_COLON] = ACTIONS(1),
    [anon_sym_localhost] = ACTIONS(1),
    [sym_port] = ACTIONS(1),
    [anon_sym_SLASH] = ACTIONS(1),
    [sym_variable_delim_start] = ACTIONS(1),
    [sym_variable_delim_end] = ACTIONS(1),
    [anon_sym_QMARK] = ACTIONS(1),
    [anon_sym_AMP] = ACTIONS(1),
    [anon_sym_EQ] = ACTIONS(1),
  },
  [1] = {
    [sym_source_file] = STATE(41),
    [sym_url_line] = STATE(24),
    [sym_url_components] = STATE(23),
    [sym_protocol] = STATE(17),
    [sym_domain_and_port] = STATE(27),
    [sym_domain] = STATE(33),
    [sym_hostname] = STATE(35),
    [sym_path] = STATE(9),
    [sym_path_param] = STATE(4),
    [sym_variable] = STATE(4),
    [aux_sym_source_file_repeat1] = STATE(3),
    [aux_sym_path_repeat1] = STATE(4),
    [ts_builtin_sym_end] = ACTIONS(3),
    [anon_sym_http] = ACTIONS(5),
    [anon_sym_https] = ACTIONS(5),
    [anon_sym_ftp] = ACTIONS(5),
    [anon_sym_ws] = ACTIONS(5),
    [anon_sym_wss] = ACTIONS(5),
    [anon_sym_COLON] = ACTIONS(7),
    [aux_sym_domain_token1] = ACTIONS(9),
    [anon_sym_localhost] = ACTIONS(11),
    [aux_sym_hostname_token1] = ACTIONS(11),
    [anon_sym_SLASH] = ACTIONS(13),
    [sym_path_segment] = ACTIONS(15),
    [sym_variable_delim_start] = ACTIONS(17),
  },
  [2] = {
    [sym_url_line] = STATE(24),
    [sym_url_components] = STATE(23),
    [sym_protocol] = STATE(17),
    [sym_domain_and_port] = STATE(27),
    [sym_domain] = STATE(33),
    [sym_hostname] = STATE(35),
    [sym_path] = STATE(9),
    [sym_path_param] = STATE(4),
    [sym_variable] = STATE(4),
    [aux_sym_source_file_repeat1] = STATE(2),
    [aux_sym_path_repeat1] = STATE(4),
    [ts_builtin_sym_end] = ACTIONS(19),
    [anon_sym_http] = ACTIONS(21),
    [anon_sym_https] = ACTIONS(21),
    [anon_sym_ftp] = ACTIONS(21),
    [anon_sym_ws] = ACTIONS(21),
    [anon_sym_wss] = ACTIONS(21),
    [anon_sym_COLON] = ACTIONS(24),
    [aux_sym_domain_token1] = ACTIONS(27),
    [anon_sym_localhost] = ACTIONS(30),
    [aux_sym_hostname_token1] = ACTIONS(30),
    [anon_sym_SLASH] = ACTIONS(33),
    [sym_path_segment] = ACTIONS(36),
    [sym_variable_delim_start] = ACTIONS(39),
  },
  [3] = {
    [sym_url_line] = STATE(24),
    [sym_url_components] = STATE(23),
    [sym_protocol] = STATE(17),
    [sym_domain_and_port] = STATE(27),
    [sym_domain] = STATE(33),
    [sym_hostname] = STATE(35),
    [sym_path] = STATE(9),
    [sym_path_param] = STATE(4),
    [sym_variable] = STATE(4),
    [aux_sym_source_file_repeat1] = STATE(2),
    [aux_sym_path_repeat1] = STATE(4),
    [ts_builtin_sym_end] = ACTIONS(42),
    [anon_sym_http] = ACTIONS(5),
    [anon_sym_https] = ACTIONS(5),
    [anon_sym_ftp] = ACTIONS(5),
    [anon_sym_ws] = ACTIONS(5),
    [anon_sym_wss] = ACTIONS(5),
    [anon_sym_COLON] = ACTIONS(7),
    [aux_sym_domain_token1] = ACTIONS(9),
    [anon_sym_localhost] = ACTIONS(11),
    [aux_sym_hostname_token1] = ACTIONS(11),
    [anon_sym_SLASH] = ACTIONS(13),
    [sym_path_segment] = ACTIONS(15),
    [sym_variable_delim_start] = ACTIONS(17),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 6,
    ACTIONS(48), 1,
      anon_sym_COLON,
    ACTIONS(54), 1,
      sym_variable_delim_start,
    ACTIONS(44), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(51), 2,
      anon_sym_SLASH,
      sym_path_segment,
    STATE(5), 3,
      sym_path_param,
      sym_variable,
      aux_sym_path_repeat1,
    ACTIONS(46), 9,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      anon_sym_QMARK,
  [31] = 6,
    ACTIONS(61), 1,
      anon_sym_COLON,
    ACTIONS(67), 1,
      sym_variable_delim_start,
    ACTIONS(57), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(64), 2,
      anon_sym_SLASH,
      sym_path_segment,
    STATE(5), 3,
      sym_path_param,
      sym_variable,
      aux_sym_path_repeat1,
    ACTIONS(59), 9,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      anon_sym_QMARK,
  [62] = 5,
    ACTIONS(70), 1,
      ts_builtin_sym_end,
    ACTIONS(74), 1,
      sym_variable_delim_start,
    ACTIONS(77), 1,
      aux_sym_value_token1,
    STATE(6), 2,
      sym_variable,
      aux_sym_value_repeat1,
    ACTIONS(72), 13,
      anon_sym_LF,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      anon_sym_COLON,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      anon_sym_SLASH,
      sym_path_segment,
      anon_sym_AMP,
  [91] = 5,
    ACTIONS(80), 1,
      ts_builtin_sym_end,
    ACTIONS(84), 1,
      sym_variable_delim_start,
    ACTIONS(86), 1,
      aux_sym_value_token1,
    STATE(6), 2,
      sym_variable,
      aux_sym_value_repeat1,
    ACTIONS(82), 13,
      anon_sym_LF,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      anon_sym_COLON,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      anon_sym_SLASH,
      sym_path_segment,
      anon_sym_AMP,
  [120] = 2,
    ACTIONS(88), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(90), 15,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      anon_sym_COLON,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
      anon_sym_QMARK,
      anon_sym_AMP,
      anon_sym_EQ,
  [142] = 4,
    ACTIONS(96), 1,
      anon_sym_QMARK,
    STATE(21), 1,
      sym_query_string,
    ACTIONS(92), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(94), 12,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      anon_sym_COLON,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
  [167] = 2,
    ACTIONS(98), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(100), 14,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      anon_sym_COLON,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
      anon_sym_AMP,
      anon_sym_EQ,
  [188] = 4,
    ACTIONS(106), 1,
      anon_sym_AMP,
    STATE(13), 1,
      aux_sym_query_string_repeat1,
    ACTIONS(102), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(104), 12,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      anon_sym_COLON,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
  [213] = 3,
    ACTIONS(112), 1,
      anon_sym_EQ,
    ACTIONS(108), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(110), 13,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      anon_sym_COLON,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
      anon_sym_AMP,
  [236] = 4,
    ACTIONS(118), 1,
      anon_sym_AMP,
    STATE(13), 1,
      aux_sym_query_string_repeat1,
    ACTIONS(114), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(116), 12,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      anon_sym_COLON,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
  [261] = 4,
    ACTIONS(106), 1,
      anon_sym_AMP,
    STATE(11), 1,
      aux_sym_query_string_repeat1,
    ACTIONS(121), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(123), 12,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      anon_sym_COLON,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
  [286] = 2,
    ACTIONS(88), 1,
      ts_builtin_sym_end,
    ACTIONS(90), 15,
      anon_sym_LF,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      anon_sym_COLON,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
      anon_sym_AMP,
      aux_sym_value_token1,
  [307] = 2,
    ACTIONS(125), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(127), 13,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      anon_sym_COLON,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
      anon_sym_QMARK,
  [327] = 12,
    ACTIONS(7), 1,
      anon_sym_COLON,
    ACTIONS(9), 1,
      aux_sym_domain_token1,
    ACTIONS(13), 1,
      anon_sym_SLASH,
    ACTIONS(15), 1,
      sym_path_segment,
    ACTIONS(17), 1,
      sym_variable_delim_start,
    STATE(9), 1,
      sym_path,
    STATE(20), 1,
      sym_url_components,
    STATE(26), 1,
      sym_domain_and_port,
    STATE(33), 1,
      sym_domain,
    STATE(35), 1,
      sym_hostname,
    ACTIONS(11), 2,
      anon_sym_localhost,
      aux_sym_hostname_token1,
    STATE(4), 3,
      sym_path_param,
      sym_variable,
      aux_sym_path_repeat1,
  [367] = 2,
    ACTIONS(129), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(131), 13,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      anon_sym_COLON,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
      anon_sym_AMP,
  [387] = 2,
    ACTIONS(114), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(116), 13,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      anon_sym_COLON,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
      anon_sym_AMP,
  [407] = 2,
    ACTIONS(133), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(135), 12,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      anon_sym_COLON,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
  [426] = 2,
    ACTIONS(137), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(139), 12,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      anon_sym_COLON,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
  [445] = 2,
    ACTIONS(141), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(143), 12,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      anon_sym_COLON,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
  [464] = 2,
    ACTIONS(145), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(147), 12,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      anon_sym_COLON,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
  [483] = 3,
    ACTIONS(149), 1,
      ts_builtin_sym_end,
    ACTIONS(151), 1,
      anon_sym_LF,
    ACTIONS(153), 12,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      anon_sym_COLON,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
  [504] = 2,
    ACTIONS(19), 4,
      ts_builtin_sym_end,
      anon_sym_COLON,
      anon_sym_SLASH,
      sym_variable_delim_start,
    ACTIONS(155), 9,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      sym_path_segment,
  [522] = 6,
    ACTIONS(7), 1,
      anon_sym_COLON,
    ACTIONS(17), 1,
      sym_variable_delim_start,
    STATE(9), 1,
      sym_path,
    STATE(22), 1,
      sym_url_components,
    ACTIONS(13), 2,
      anon_sym_SLASH,
      sym_path_segment,
    STATE(4), 3,
      sym_path_param,
      sym_variable,
      aux_sym_path_repeat1,
  [544] = 6,
    ACTIONS(7), 1,
      anon_sym_COLON,
    ACTIONS(17), 1,
      sym_variable_delim_start,
    STATE(9), 1,
      sym_path,
    STATE(20), 1,
      sym_url_components,
    ACTIONS(13), 2,
      anon_sym_SLASH,
      sym_path_segment,
    STATE(4), 3,
      sym_path_param,
      sym_variable,
      aux_sym_path_repeat1,
  [566] = 2,
    ACTIONS(157), 3,
      anon_sym_COLON,
      anon_sym_SLASH,
      sym_variable_delim_start,
    ACTIONS(159), 4,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      sym_path_segment,
  [578] = 5,
    ACTIONS(17), 1,
      sym_variable_delim_start,
    ACTIONS(161), 1,
      aux_sym_key_token1,
    STATE(10), 1,
      sym_variable,
    STATE(12), 1,
      sym_key,
    STATE(14), 1,
      sym_query_param,
  [594] = 4,
    ACTIONS(84), 1,
      sym_variable_delim_start,
    ACTIONS(163), 1,
      aux_sym_value_token1,
    STATE(18), 1,
      sym_value,
    STATE(7), 2,
      sym_variable,
      aux_sym_value_repeat1,
  [608] = 5,
    ACTIONS(17), 1,
      sym_variable_delim_start,
    ACTIONS(161), 1,
      aux_sym_key_token1,
    STATE(10), 1,
      sym_variable,
    STATE(12), 1,
      sym_key,
    STATE(19), 1,
      sym_query_param,
  [624] = 1,
    ACTIONS(165), 4,
      anon_sym_COLON,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
  [631] = 2,
    ACTIONS(167), 1,
      anon_sym_COLON,
    ACTIONS(170), 3,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
  [640] = 1,
    ACTIONS(172), 4,
      anon_sym_COLON,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
  [647] = 1,
    ACTIONS(174), 4,
      anon_sym_COLON,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
  [654] = 1,
    ACTIONS(176), 1,
      anon_sym_COLON_SLASH_SLASH,
  [658] = 1,
    ACTIONS(178), 1,
      sym_port,
  [662] = 1,
    ACTIONS(180), 1,
      sym_variable_name,
  [666] = 1,
    ACTIONS(182), 1,
      sym_variable_delim_end,
  [670] = 1,
    ACTIONS(184), 1,
      aux_sym_path_param_token1,
  [674] = 1,
    ACTIONS(186), 1,
      ts_builtin_sym_end,
  [678] = 1,
    ACTIONS(188), 1,
      sym_variable_delim_end,
  [682] = 1,
    ACTIONS(190), 1,
      sym_variable_name,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(4)] = 0,
  [SMALL_STATE(5)] = 31,
  [SMALL_STATE(6)] = 62,
  [SMALL_STATE(7)] = 91,
  [SMALL_STATE(8)] = 120,
  [SMALL_STATE(9)] = 142,
  [SMALL_STATE(10)] = 167,
  [SMALL_STATE(11)] = 188,
  [SMALL_STATE(12)] = 213,
  [SMALL_STATE(13)] = 236,
  [SMALL_STATE(14)] = 261,
  [SMALL_STATE(15)] = 286,
  [SMALL_STATE(16)] = 307,
  [SMALL_STATE(17)] = 327,
  [SMALL_STATE(18)] = 367,
  [SMALL_STATE(19)] = 387,
  [SMALL_STATE(20)] = 407,
  [SMALL_STATE(21)] = 426,
  [SMALL_STATE(22)] = 445,
  [SMALL_STATE(23)] = 464,
  [SMALL_STATE(24)] = 483,
  [SMALL_STATE(25)] = 504,
  [SMALL_STATE(26)] = 522,
  [SMALL_STATE(27)] = 544,
  [SMALL_STATE(28)] = 566,
  [SMALL_STATE(29)] = 578,
  [SMALL_STATE(30)] = 594,
  [SMALL_STATE(31)] = 608,
  [SMALL_STATE(32)] = 624,
  [SMALL_STATE(33)] = 631,
  [SMALL_STATE(34)] = 640,
  [SMALL_STATE(35)] = 647,
  [SMALL_STATE(36)] = 654,
  [SMALL_STATE(37)] = 658,
  [SMALL_STATE(38)] = 662,
  [SMALL_STATE(39)] = 666,
  [SMALL_STATE(40)] = 670,
  [SMALL_STATE(41)] = 674,
  [SMALL_STATE(42)] = 678,
  [SMALL_STATE(43)] = 682,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0, 0, 0),
  [5] = {.entry = {.count = 1, .reusable = false}}, SHIFT(36),
  [7] = {.entry = {.count = 1, .reusable = true}}, SHIFT(40),
  [9] = {.entry = {.count = 1, .reusable = false}}, SHIFT(35),
  [11] = {.entry = {.count = 1, .reusable = false}}, SHIFT(34),
  [13] = {.entry = {.count = 1, .reusable = true}}, SHIFT(4),
  [15] = {.entry = {.count = 1, .reusable = false}}, SHIFT(4),
  [17] = {.entry = {.count = 1, .reusable = true}}, SHIFT(38),
  [19] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0),
  [21] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(36),
  [24] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(40),
  [27] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(35),
  [30] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(34),
  [33] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(4),
  [36] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(4),
  [39] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(38),
  [42] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, 0, 0),
  [44] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_path, 1, 0, 0),
  [46] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_path, 1, 0, 0),
  [48] = {.entry = {.count = 2, .reusable = false}}, REDUCE(sym_path, 1, 0, 0), SHIFT(40),
  [51] = {.entry = {.count = 2, .reusable = false}}, REDUCE(sym_path, 1, 0, 0), SHIFT(5),
  [54] = {.entry = {.count = 2, .reusable = false}}, REDUCE(sym_path, 1, 0, 0), SHIFT(38),
  [57] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_path_repeat1, 2, 0, 0),
  [59] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_path_repeat1, 2, 0, 0),
  [61] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_path_repeat1, 2, 0, 0), SHIFT_REPEAT(40),
  [64] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_path_repeat1, 2, 0, 0), SHIFT_REPEAT(5),
  [67] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_path_repeat1, 2, 0, 0), SHIFT_REPEAT(38),
  [70] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_value_repeat1, 2, 0, 0),
  [72] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_value_repeat1, 2, 0, 0),
  [74] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_value_repeat1, 2, 0, 0), SHIFT_REPEAT(43),
  [77] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_value_repeat1, 2, 0, 0), SHIFT_REPEAT(6),
  [80] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_value, 1, 0, 0),
  [82] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_value, 1, 0, 0),
  [84] = {.entry = {.count = 1, .reusable = false}}, SHIFT(43),
  [86] = {.entry = {.count = 1, .reusable = false}}, SHIFT(6),
  [88] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_variable, 3, 0, 0),
  [90] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_variable, 3, 0, 0),
  [92] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_url_components, 1, 0, 0),
  [94] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_url_components, 1, 0, 0),
  [96] = {.entry = {.count = 1, .reusable = false}}, SHIFT(29),
  [98] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_key, 1, 0, 0),
  [100] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_key, 1, 0, 0),
  [102] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_query_string, 3, 0, 0),
  [104] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_query_string, 3, 0, 0),
  [106] = {.entry = {.count = 1, .reusable = false}}, SHIFT(31),
  [108] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_query_param, 1, 0, 0),
  [110] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_query_param, 1, 0, 0),
  [112] = {.entry = {.count = 1, .reusable = false}}, SHIFT(30),
  [114] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_query_string_repeat1, 2, 0, 0),
  [116] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_query_string_repeat1, 2, 0, 0),
  [118] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_query_string_repeat1, 2, 0, 0), SHIFT_REPEAT(31),
  [121] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_query_string, 2, 0, 0),
  [123] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_query_string, 2, 0, 0),
  [125] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_path_param, 2, 0, 0),
  [127] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_path_param, 2, 0, 0),
  [129] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_query_param, 3, 0, 0),
  [131] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_query_param, 3, 0, 0),
  [133] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_url_line, 2, 0, 0),
  [135] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_url_line, 2, 0, 0),
  [137] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_url_components, 2, 0, 0),
  [139] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_url_components, 2, 0, 0),
  [141] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_url_line, 3, 0, 0),
  [143] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_url_line, 3, 0, 0),
  [145] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_url_line, 1, 0, 0),
  [147] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_url_line, 1, 0, 0),
  [149] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 1, 0, 0),
  [151] = {.entry = {.count = 1, .reusable = true}}, SHIFT(25),
  [153] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 1, 0, 0),
  [155] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0),
  [157] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_protocol, 2, 0, 0),
  [159] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_protocol, 2, 0, 0),
  [161] = {.entry = {.count = 1, .reusable = true}}, SHIFT(10),
  [163] = {.entry = {.count = 1, .reusable = true}}, SHIFT(7),
  [165] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_domain_and_port, 3, 0, 0),
  [167] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym_domain_and_port, 1, 0, 0), SHIFT(37),
  [170] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_domain_and_port, 1, 0, 0),
  [172] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_hostname, 1, 0, 0),
  [174] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_domain, 1, 0, 0),
  [176] = {.entry = {.count = 1, .reusable = true}}, SHIFT(28),
  [178] = {.entry = {.count = 1, .reusable = true}}, SHIFT(32),
  [180] = {.entry = {.count = 1, .reusable = true}}, SHIFT(39),
  [182] = {.entry = {.count = 1, .reusable = true}}, SHIFT(8),
  [184] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [186] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [188] = {.entry = {.count = 1, .reusable = true}}, SHIFT(15),
  [190] = {.entry = {.count = 1, .reusable = true}}, SHIFT(42),
};

#ifdef __cplusplus
extern "C" {
#endif
#ifdef TREE_SITTER_HIDE_SYMBOLS
#define TS_PUBLIC
#elif defined(_WIN32)
#define TS_PUBLIC __declspec(dllexport)
#else
#define TS_PUBLIC __attribute__((visibility("default")))
#endif

TS_PUBLIC const TSLanguage *tree_sitter_url(void) {
  static const TSLanguage language = {
    .version = LANGUAGE_VERSION,
    .symbol_count = SYMBOL_COUNT,
    .alias_count = ALIAS_COUNT,
    .token_count = TOKEN_COUNT,
    .external_token_count = EXTERNAL_TOKEN_COUNT,
    .state_count = STATE_COUNT,
    .large_state_count = LARGE_STATE_COUNT,
    .production_id_count = PRODUCTION_ID_COUNT,
    .field_count = FIELD_COUNT,
    .max_alias_sequence_length = MAX_ALIAS_SEQUENCE_LENGTH,
    .parse_table = &ts_parse_table[0][0],
    .small_parse_table = ts_small_parse_table,
    .small_parse_table_map = ts_small_parse_table_map,
    .parse_actions = ts_parse_actions,
    .symbol_names = ts_symbol_names,
    .symbol_metadata = ts_symbol_metadata,
    .public_symbol_map = ts_symbol_map,
    .alias_map = ts_non_terminal_alias_map,
    .alias_sequences = &ts_alias_sequences[0][0],
    .lex_modes = ts_lex_modes,
    .lex_fn = ts_lex,
    .primary_state_ids = ts_primary_state_ids,
  };
  return &language;
}
#ifdef __cplusplus
}
#endif
