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
#define STATE_COUNT 40
#define LARGE_STATE_COUNT 4
#define SYMBOL_COUNT 41
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
  [15] = 15,
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
      if (eof) ADVANCE(27);
      ADVANCE_MAP(
        '&', 334,
        '/', 120,
        ':', 41,
        '=', 335,
        '?', 333,
        'f', 16,
        'h', 19,
        'l', 10,
        'w', 14,
        '{', 20,
        '}', 22,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(0);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(119);
      END_STATE();
    case 1:
      if (lookahead == '/') ADVANCE(120);
      if (lookahead == ':') ADVANCE(40);
      if (lookahead == 'l') ADVANCE(121);
      if (lookahead == '{') ADVANCE(20);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(1);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(126);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(125);
      if (lookahead != 0 &&
          lookahead != '&' &&
          lookahead != '=' &&
          lookahead != '?' &&
          lookahead != '}') ADVANCE(327);
      END_STATE();
    case 2:
      if (lookahead == '/') ADVANCE(120);
      if (lookahead == ':') ADVANCE(40);
      if (lookahead == '{') ADVANCE(20);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(2);
      if (lookahead != 0 &&
          lookahead != '&' &&
          lookahead != '=' &&
          lookahead != '?' &&
          lookahead != '}') ADVANCE(327);
      END_STATE();
    case 3:
      if (lookahead == '/') ADVANCE(4);
      END_STATE();
    case 4:
      if (lookahead == '/') ADVANCE(39);
      END_STATE();
    case 5:
      if (lookahead == ':') ADVANCE(3);
      if (lookahead == '{') ADVANCE(20);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(5);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(336);
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
      if (lookahead == 'p') ADVANCE(33);
      END_STATE();
    case 13:
      if (lookahead == 'p') ADVANCE(30);
      END_STATE();
    case 14:
      if (lookahead == 's') ADVANCE(36);
      END_STATE();
    case 15:
      if (lookahead == 's') ADVANCE(17);
      END_STATE();
    case 16:
      if (lookahead == 't') ADVANCE(12);
      END_STATE();
    case 17:
      if (lookahead == 't') ADVANCE(113);
      END_STATE();
    case 18:
      if (lookahead == 't') ADVANCE(13);
      END_STATE();
    case 19:
      if (lookahead == 't') ADVANCE(18);
      END_STATE();
    case 20:
      if (lookahead == '{') ADVANCE(329);
      END_STATE();
    case 21:
      if (lookahead == '{') ADVANCE(338);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(337);
      if (lookahead != 0 &&
          lookahead != '&') ADVANCE(339);
      END_STATE();
    case 22:
      if (lookahead == '}') ADVANCE(331);
      END_STATE();
    case 23:
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(23);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 24:
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(24);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(332);
      END_STATE();
    case 25:
      if (eof) ADVANCE(27);
      ADVANCE_MAP(
        '\n', 28,
        '&', 334,
        '/', 120,
        ':', 40,
        '=', 335,
        '?', 333,
        'f', 123,
        'h', 124,
        'l', 121,
        'w', 122,
        '{', 20,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(25);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(126);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(125);
      if (lookahead != 0 &&
          lookahead != '}') ADVANCE(327);
      END_STATE();
    case 26:
      if (eof) ADVANCE(27);
      if (lookahead == '/') ADVANCE(120);
      if (lookahead == ':') ADVANCE(40);
      if (lookahead == 'f') ADVANCE(123);
      if (lookahead == 'h') ADVANCE(124);
      if (lookahead == 'l') ADVANCE(121);
      if (lookahead == 'w') ADVANCE(122);
      if (lookahead == '{') ADVANCE(20);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(26);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(126);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(125);
      if (lookahead != 0 &&
          lookahead != '&' &&
          lookahead != '=' &&
          lookahead != '?' &&
          lookahead != '}') ADVANCE(327);
      END_STATE();
    case 27:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 28:
      ACCEPT_TOKEN(anon_sym_LF);
      if (lookahead == '\n') ADVANCE(28);
      END_STATE();
    case 29:
      ACCEPT_TOKEN(anon_sym_http);
      if (lookahead == '-') ADVANCE(296);
      if (lookahead == '.') ADVANCE(324);
      if (lookahead == 's') ADVANCE(32);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(295);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 30:
      ACCEPT_TOKEN(anon_sym_http);
      if (lookahead == 's') ADVANCE(31);
      END_STATE();
    case 31:
      ACCEPT_TOKEN(anon_sym_https);
      END_STATE();
    case 32:
      ACCEPT_TOKEN(anon_sym_https);
      if (lookahead == '-') ADVANCE(300);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(299);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 33:
      ACCEPT_TOKEN(anon_sym_ftp);
      END_STATE();
    case 34:
      ACCEPT_TOKEN(anon_sym_ftp);
      if (lookahead == '-') ADVANCE(310);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(309);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 35:
      ACCEPT_TOKEN(anon_sym_ws);
      if (lookahead == '-') ADVANCE(306);
      if (lookahead == '.') ADVANCE(324);
      if (lookahead == 's') ADVANCE(38);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(304);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 36:
      ACCEPT_TOKEN(anon_sym_ws);
      if (lookahead == 's') ADVANCE(37);
      END_STATE();
    case 37:
      ACCEPT_TOKEN(anon_sym_wss);
      END_STATE();
    case 38:
      ACCEPT_TOKEN(anon_sym_wss);
      if (lookahead == '-') ADVANCE(310);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(309);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 39:
      ACCEPT_TOKEN(anon_sym_COLON_SLASH_SLASH);
      END_STATE();
    case 40:
      ACCEPT_TOKEN(anon_sym_COLON);
      END_STATE();
    case 41:
      ACCEPT_TOKEN(anon_sym_COLON);
      if (lookahead == '/') ADVANCE(4);
      END_STATE();
    case 42:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(318);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(107);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(108);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 43:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(318);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(108);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 44:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(318);
      if (lookahead == '.') ADVANCE(322);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(109);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(108);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 45:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(318);
      if (lookahead == '.') ADVANCE(323);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(110);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(108);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 46:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(326);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(111);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 47:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(135);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(48);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 48:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(131);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(46);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 49:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(141);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(50);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 50:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(134);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(47);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 51:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(147);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(52);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 52:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(138);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(49);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 53:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(153);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(54);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 54:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(144);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(51);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 55:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(159);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(56);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 56:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(150);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(53);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 57:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(165);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(58);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 58:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(156);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(55);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 59:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(171);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(60);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 60:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(162);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(57);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 61:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(177);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(62);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 62:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(168);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(59);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 63:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(183);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 64:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(174);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(61);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 65:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(189);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(66);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 66:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(180);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(63);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 67:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(195);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(68);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 68:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(186);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(65);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 69:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(201);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(70);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 70:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(192);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(67);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 71:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(207);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(72);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 72:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(198);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(69);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 73:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(213);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(74);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 74:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(204);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(71);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 75:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(219);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(76);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 76:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(210);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(73);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 77:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(225);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(78);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 78:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(216);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(75);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 79:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(231);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(80);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 80:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(222);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(77);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 81:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(237);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(82);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 82:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(228);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(79);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 83:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(243);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(84);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 84:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(234);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(81);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 85:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(249);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(86);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 86:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(240);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(83);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 87:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(255);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(88);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 88:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(246);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 89:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(261);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(90);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 90:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(252);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(87);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 91:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(267);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(92);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 92:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(258);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(89);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 93:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(273);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(94);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 94:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(264);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(91);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 95:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(279);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(96);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 96:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(270);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(93);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 97:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(285);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(98);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 98:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(276);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(95);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 99:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(293);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(100);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 100:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(282);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(97);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 101:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(301);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(102);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 102:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(289);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(99);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 103:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(311);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(106);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 104:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(311);
      if (lookahead == '.') ADVANCE(322);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(106);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 105:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(311);
      if (lookahead == '.') ADVANCE(323);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(106);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 106:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(297);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(101);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 107:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(307);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(103);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(103);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 108:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(307);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(103);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 109:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(307);
      if (lookahead == '.') ADVANCE(322);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(104);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(103);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 110:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '-') ADVANCE(307);
      if (lookahead == '.') ADVANCE(323);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(105);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(103);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 111:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(112);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 112:
      ACCEPT_TOKEN(aux_sym_domain_token1);
      if (lookahead == '.') ADVANCE(324);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 113:
      ACCEPT_TOKEN(anon_sym_localhost);
      END_STATE();
    case 114:
      ACCEPT_TOKEN(anon_sym_localhost);
      if (lookahead == '-') ADVANCE(284);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(283);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 115:
      ACCEPT_TOKEN(sym_port);
      END_STATE();
    case 116:
      ACCEPT_TOKEN(sym_port);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(115);
      END_STATE();
    case 117:
      ACCEPT_TOKEN(sym_port);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(116);
      END_STATE();
    case 118:
      ACCEPT_TOKEN(sym_port);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(117);
      END_STATE();
    case 119:
      ACCEPT_TOKEN(sym_port);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(118);
      END_STATE();
    case 120:
      ACCEPT_TOKEN(anon_sym_SLASH);
      END_STATE();
    case 121:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(317);
      if (lookahead == '.') ADVANCE(324);
      if (lookahead == 'o') ADVANCE(312);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(315);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 122:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(317);
      if (lookahead == '.') ADVANCE(324);
      if (lookahead == 's') ADVANCE(35);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(315);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 123:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(317);
      if (lookahead == '.') ADVANCE(324);
      if (lookahead == 't') ADVANCE(313);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(315);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 124:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(317);
      if (lookahead == '.') ADVANCE(324);
      if (lookahead == 't') ADVANCE(314);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(315);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 125:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(317);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(315);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 126:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(317);
      if (lookahead == '.') ADVANCE(321);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(316);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(315);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 127:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(325);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(319);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 128:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(325);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(319);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 129:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(133);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(132);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 130:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(133);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(132);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 131:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(326);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(111);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 132:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(128);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(127);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 133:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(128);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(127);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 134:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(135);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(48);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 135:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(131);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(46);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 136:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(140);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(139);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 137:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(140);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(139);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 138:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(141);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(50);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 139:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(130);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(129);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 140:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(130);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(129);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 141:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(134);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(47);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 142:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(146);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(145);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 143:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(146);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(145);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 144:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(147);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(52);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 145:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(137);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(136);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 146:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(137);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(136);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 147:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(138);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(49);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 148:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(152);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(151);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 149:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(152);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(151);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 150:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(153);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(54);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 151:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(143);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(142);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 152:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(143);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(142);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 153:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(144);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(51);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 154:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(158);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(157);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 155:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(158);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(157);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 156:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(159);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(56);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 157:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(149);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(148);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 158:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(149);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(148);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 159:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(150);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(53);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 160:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(164);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 161:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(164);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(163);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 162:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(165);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(58);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 163:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(155);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(154);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 164:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(155);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(154);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 165:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(156);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(55);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 166:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(170);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(169);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 167:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(170);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(169);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 168:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(171);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(60);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 169:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(161);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(160);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 170:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(161);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(160);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 171:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(162);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(57);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 172:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(176);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(175);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 173:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(176);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(175);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 174:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(177);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(62);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 175:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(167);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(166);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 176:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(167);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(166);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 177:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(168);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(59);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 178:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(182);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(181);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 179:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(182);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(181);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 180:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(183);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(64);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 181:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(173);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(172);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 182:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(173);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(172);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 183:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(174);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(61);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 184:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(188);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(187);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 185:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(188);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(187);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 186:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(189);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(66);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 187:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(179);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(178);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 188:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(179);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(178);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 189:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(180);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(63);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 190:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(194);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(193);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 191:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(194);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(193);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 192:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(195);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(68);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 193:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(185);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(184);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 194:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(185);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(184);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 195:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(186);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(65);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 196:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(200);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(199);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 197:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(200);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(199);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 198:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(201);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(70);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 199:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(191);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(190);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 200:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(191);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(190);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 201:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(192);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(67);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 202:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(206);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(205);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 203:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(206);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(205);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 204:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(207);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(72);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 205:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(197);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(196);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 206:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(197);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(196);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 207:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(198);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(69);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 208:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(212);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(211);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 209:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(212);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(211);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 210:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(74);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 211:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(203);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(202);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 212:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(203);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(202);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 213:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(204);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(71);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 214:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(218);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(217);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 215:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(218);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(217);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 216:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(219);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(76);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 217:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(209);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(208);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 218:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(209);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(208);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 219:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(210);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(73);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 220:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(224);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(223);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 221:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(224);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(223);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 222:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(225);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(78);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 223:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(215);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(214);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 224:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(215);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(214);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 225:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(216);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(75);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 226:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(230);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(229);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 227:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(230);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(229);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 228:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(231);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(80);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 229:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(221);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(220);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 230:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(221);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(220);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 231:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(222);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(77);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 232:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(236);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(235);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 233:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(236);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(235);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 234:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(237);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(82);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 235:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(227);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(226);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 236:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(227);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(226);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 237:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(228);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(79);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 238:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(242);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(241);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 239:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(242);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(241);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 240:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(243);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(84);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 241:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(233);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(232);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 242:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(233);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(232);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 243:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(234);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(81);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 244:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(248);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(247);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 245:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(248);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(247);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 246:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(249);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(86);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 247:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(239);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(238);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 248:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(239);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(238);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 249:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(240);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(83);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 250:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(254);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(253);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 251:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(254);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(253);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 252:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(255);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(88);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 253:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(245);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(244);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 254:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(245);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(244);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 255:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(246);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(85);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 256:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(260);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(259);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 257:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(260);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(259);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 258:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(261);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(90);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 259:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(251);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(250);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 260:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(251);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(250);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 261:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(252);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(87);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 262:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(266);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(265);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 263:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(266);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(265);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 264:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(267);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(92);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 265:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(257);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(256);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 266:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(257);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(256);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 267:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(258);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(89);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 268:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(272);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(271);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 269:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(272);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(271);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 270:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(273);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(94);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 271:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(263);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(262);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 272:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(263);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(262);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 273:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(264);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(91);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 274:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(278);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(277);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 275:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(278);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(277);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 276:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(279);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(96);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 277:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(269);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(268);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 278:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(269);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(268);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 279:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(270);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(93);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 280:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(284);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(283);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 281:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(284);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(283);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 282:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(285);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(98);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 283:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(275);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(274);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 284:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(275);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(274);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 285:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(276);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(95);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 286:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(292);
      if (lookahead == '.') ADVANCE(324);
      if (lookahead == 's') ADVANCE(290);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(291);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 287:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(292);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(291);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 288:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(292);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(291);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 289:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(293);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(100);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 290:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(281);
      if (lookahead == '.') ADVANCE(324);
      if (lookahead == 't') ADVANCE(114);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(280);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 291:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(281);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(280);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 292:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(281);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(280);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 293:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(282);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(97);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 294:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(300);
      if (lookahead == '.') ADVANCE(324);
      if (lookahead == 'h') ADVANCE(298);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(299);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 295:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(300);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(299);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 296:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(300);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(299);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 297:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(301);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(102);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 298:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(288);
      if (lookahead == '.') ADVANCE(324);
      if (lookahead == 'o') ADVANCE(286);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(287);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 299:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(288);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(287);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 300:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(288);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(287);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 301:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(289);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(99);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 302:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(310);
      if (lookahead == '.') ADVANCE(324);
      if (lookahead == 'a') ADVANCE(308);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(309);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 303:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(310);
      if (lookahead == '.') ADVANCE(324);
      if (lookahead == 'p') ADVANCE(29);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(309);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 304:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(310);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(309);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 305:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(310);
      if (lookahead == '.') ADVANCE(321);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(309);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 306:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(310);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(309);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 307:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(311);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(106);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 308:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(296);
      if (lookahead == '.') ADVANCE(324);
      if (lookahead == 'l') ADVANCE(294);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(295);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 309:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(296);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(295);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 310:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(296);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(295);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 311:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(297);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(101);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 312:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(306);
      if (lookahead == '.') ADVANCE(324);
      if (lookahead == 'c') ADVANCE(302);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(304);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 313:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(306);
      if (lookahead == '.') ADVANCE(324);
      if (lookahead == 'p') ADVANCE(34);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(304);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 314:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(306);
      if (lookahead == '.') ADVANCE(324);
      if (lookahead == 't') ADVANCE(303);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(304);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 315:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(306);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(304);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 316:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(306);
      if (lookahead == '.') ADVANCE(321);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(305);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(304);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 317:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(306);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(304);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 318:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '-') ADVANCE(307);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(103);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 319:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '.') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(320);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 320:
      ACCEPT_TOKEN(sym_path_segment);
      if (lookahead == '.') ADVANCE(324);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 321:
      ACCEPT_TOKEN(sym_path_segment);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(45);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(43);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 322:
      ACCEPT_TOKEN(sym_path_segment);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(42);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(43);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 323:
      ACCEPT_TOKEN(sym_path_segment);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(44);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(43);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 324:
      ACCEPT_TOKEN(sym_path_segment);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(43);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 325:
      ACCEPT_TOKEN(sym_path_segment);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(320);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 326:
      ACCEPT_TOKEN(sym_path_segment);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(112);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 327:
      ACCEPT_TOKEN(sym_path_segment);
      if ((!eof && set_contains(sym_path_segment_character_set_1, 10, lookahead))) ADVANCE(327);
      END_STATE();
    case 328:
      ACCEPT_TOKEN(aux_sym_path_param_token1);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(328);
      END_STATE();
    case 329:
      ACCEPT_TOKEN(sym_variable_delim_start);
      END_STATE();
    case 330:
      ACCEPT_TOKEN(sym_variable_delim_start);
      if (lookahead != 0 &&
          lookahead != '&') ADVANCE(339);
      END_STATE();
    case 331:
      ACCEPT_TOKEN(sym_variable_delim_end);
      END_STATE();
    case 332:
      ACCEPT_TOKEN(sym_variable_name);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(332);
      END_STATE();
    case 333:
      ACCEPT_TOKEN(anon_sym_QMARK);
      END_STATE();
    case 334:
      ACCEPT_TOKEN(anon_sym_AMP);
      END_STATE();
    case 335:
      ACCEPT_TOKEN(anon_sym_EQ);
      END_STATE();
    case 336:
      ACCEPT_TOKEN(aux_sym_key_token1);
      if (lookahead == '-' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(336);
      END_STATE();
    case 337:
      ACCEPT_TOKEN(aux_sym_value_token1);
      if (lookahead == '{') ADVANCE(338);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(337);
      if (lookahead != 0 &&
          lookahead != '&') ADVANCE(339);
      END_STATE();
    case 338:
      ACCEPT_TOKEN(aux_sym_value_token1);
      if (lookahead == '{') ADVANCE(330);
      if (lookahead != 0 &&
          lookahead != '&') ADVANCE(339);
      END_STATE();
    case 339:
      ACCEPT_TOKEN(aux_sym_value_token1);
      if (lookahead != 0 &&
          lookahead != '&') ADVANCE(339);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 26},
  [2] = {.lex_state = 26},
  [3] = {.lex_state = 26},
  [4] = {.lex_state = 25},
  [5] = {.lex_state = 25},
  [6] = {.lex_state = 25},
  [7] = {.lex_state = 25},
  [8] = {.lex_state = 25},
  [9] = {.lex_state = 25},
  [10] = {.lex_state = 25},
  [11] = {.lex_state = 25},
  [12] = {.lex_state = 25},
  [13] = {.lex_state = 25},
  [14] = {.lex_state = 1},
  [15] = {.lex_state = 25},
  [16] = {.lex_state = 25},
  [17] = {.lex_state = 25},
  [18] = {.lex_state = 25},
  [19] = {.lex_state = 25},
  [20] = {.lex_state = 25},
  [21] = {.lex_state = 25},
  [22] = {.lex_state = 25},
  [23] = {.lex_state = 26},
  [24] = {.lex_state = 2},
  [25] = {.lex_state = 2},
  [26] = {.lex_state = 1},
  [27] = {.lex_state = 5},
  [28] = {.lex_state = 5},
  [29] = {.lex_state = 2},
  [30] = {.lex_state = 2},
  [31] = {.lex_state = 2},
  [32] = {.lex_state = 21},
  [33] = {.lex_state = 2},
  [34] = {.lex_state = 0},
  [35] = {.lex_state = 0},
  [36] = {.lex_state = 5},
  [37] = {.lex_state = 23},
  [38] = {.lex_state = 24},
  [39] = {.lex_state = 0},
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
    [sym_source_file] = STATE(35),
    [sym_url_line] = STATE(21),
    [sym_url_components] = STATE(20),
    [sym_protocol] = STATE(14),
    [sym_domain_and_port] = STATE(25),
    [sym_domain] = STATE(30),
    [sym_hostname] = STATE(31),
    [sym_path] = STATE(7),
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
    [sym_url_line] = STATE(21),
    [sym_url_components] = STATE(20),
    [sym_protocol] = STATE(14),
    [sym_domain_and_port] = STATE(25),
    [sym_domain] = STATE(30),
    [sym_hostname] = STATE(31),
    [sym_path] = STATE(7),
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
    [sym_url_line] = STATE(21),
    [sym_url_components] = STATE(20),
    [sym_protocol] = STATE(14),
    [sym_domain_and_port] = STATE(25),
    [sym_domain] = STATE(30),
    [sym_hostname] = STATE(31),
    [sym_path] = STATE(7),
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
  [62] = 2,
    ACTIONS(70), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(72), 15,
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
  [84] = 4,
    ACTIONS(78), 1,
      anon_sym_QMARK,
    STATE(19), 1,
      sym_query_string,
    ACTIONS(74), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(76), 12,
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
  [109] = 2,
    ACTIONS(80), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(82), 14,
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
  [130] = 4,
    ACTIONS(88), 1,
      anon_sym_AMP,
    STATE(11), 1,
      aux_sym_query_string_repeat1,
    ACTIONS(84), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(86), 12,
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
  [155] = 3,
    ACTIONS(94), 1,
      anon_sym_EQ,
    ACTIONS(90), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(92), 13,
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
  [178] = 4,
    ACTIONS(88), 1,
      anon_sym_AMP,
    STATE(12), 1,
      aux_sym_query_string_repeat1,
    ACTIONS(96), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(98), 12,
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
  [203] = 4,
    ACTIONS(104), 1,
      anon_sym_AMP,
    STATE(12), 1,
      aux_sym_query_string_repeat1,
    ACTIONS(100), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(102), 12,
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
  [228] = 2,
    ACTIONS(107), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(109), 13,
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
  [248] = 12,
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
    STATE(7), 1,
      sym_path,
    STATE(18), 1,
      sym_url_components,
    STATE(24), 1,
      sym_domain_and_port,
    STATE(30), 1,
      sym_domain,
    STATE(31), 1,
      sym_hostname,
    ACTIONS(11), 2,
      anon_sym_localhost,
      aux_sym_hostname_token1,
    STATE(4), 3,
      sym_path_param,
      sym_variable,
      aux_sym_path_repeat1,
  [288] = 2,
    ACTIONS(100), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(102), 13,
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
  [308] = 2,
    ACTIONS(111), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(113), 13,
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
  [328] = 2,
    ACTIONS(115), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(117), 13,
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
  [348] = 2,
    ACTIONS(119), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(121), 12,
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
  [367] = 2,
    ACTIONS(123), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(125), 12,
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
  [386] = 2,
    ACTIONS(127), 2,
      ts_builtin_sym_end,
      anon_sym_LF,
    ACTIONS(129), 12,
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
  [405] = 3,
    ACTIONS(131), 1,
      ts_builtin_sym_end,
    ACTIONS(133), 1,
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
    ACTIONS(19), 4,
      ts_builtin_sym_end,
      anon_sym_COLON,
      anon_sym_SLASH,
      sym_variable_delim_start,
    ACTIONS(141), 9,
      anon_sym_http,
      anon_sym_https,
      anon_sym_ftp,
      anon_sym_ws,
      anon_sym_wss,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      sym_path_segment,
  [463] = 6,
    ACTIONS(7), 1,
      anon_sym_COLON,
    ACTIONS(17), 1,
      sym_variable_delim_start,
    STATE(7), 1,
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
  [485] = 6,
    ACTIONS(7), 1,
      anon_sym_COLON,
    ACTIONS(17), 1,
      sym_variable_delim_start,
    STATE(7), 1,
      sym_path,
    STATE(18), 1,
      sym_url_components,
    ACTIONS(13), 2,
      anon_sym_SLASH,
      sym_path_segment,
    STATE(4), 3,
      sym_path_param,
      sym_variable,
      aux_sym_path_repeat1,
  [507] = 2,
    ACTIONS(143), 3,
      anon_sym_COLON,
      anon_sym_SLASH,
      sym_variable_delim_start,
    ACTIONS(145), 4,
      aux_sym_domain_token1,
      anon_sym_localhost,
      aux_sym_hostname_token1,
      sym_path_segment,
  [519] = 5,
    ACTIONS(17), 1,
      sym_variable_delim_start,
    ACTIONS(147), 1,
      aux_sym_key_token1,
    STATE(8), 1,
      sym_variable,
    STATE(9), 1,
      sym_query_param,
    STATE(10), 1,
      sym_key,
  [535] = 5,
    ACTIONS(17), 1,
      sym_variable_delim_start,
    ACTIONS(147), 1,
      aux_sym_key_token1,
    STATE(8), 1,
      sym_variable,
    STATE(10), 1,
      sym_key,
    STATE(15), 1,
      sym_query_param,
  [551] = 1,
    ACTIONS(149), 4,
      anon_sym_COLON,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
  [558] = 2,
    ACTIONS(151), 1,
      anon_sym_COLON,
    ACTIONS(154), 3,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
  [567] = 1,
    ACTIONS(156), 4,
      anon_sym_COLON,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
  [574] = 4,
    ACTIONS(158), 1,
      sym_variable_delim_start,
    ACTIONS(160), 1,
      aux_sym_value_token1,
    STATE(16), 1,
      sym_variable,
    STATE(17), 1,
      sym_value,
  [587] = 1,
    ACTIONS(162), 4,
      anon_sym_COLON,
      anon_sym_SLASH,
      sym_path_segment,
      sym_variable_delim_start,
  [594] = 1,
    ACTIONS(164), 1,
      sym_port,
  [598] = 1,
    ACTIONS(166), 1,
      ts_builtin_sym_end,
  [602] = 1,
    ACTIONS(168), 1,
      anon_sym_COLON_SLASH_SLASH,
  [606] = 1,
    ACTIONS(170), 1,
      aux_sym_path_param_token1,
  [610] = 1,
    ACTIONS(172), 1,
      sym_variable_name,
  [614] = 1,
    ACTIONS(174), 1,
      sym_variable_delim_end,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(4)] = 0,
  [SMALL_STATE(5)] = 31,
  [SMALL_STATE(6)] = 62,
  [SMALL_STATE(7)] = 84,
  [SMALL_STATE(8)] = 109,
  [SMALL_STATE(9)] = 130,
  [SMALL_STATE(10)] = 155,
  [SMALL_STATE(11)] = 178,
  [SMALL_STATE(12)] = 203,
  [SMALL_STATE(13)] = 228,
  [SMALL_STATE(14)] = 248,
  [SMALL_STATE(15)] = 288,
  [SMALL_STATE(16)] = 308,
  [SMALL_STATE(17)] = 328,
  [SMALL_STATE(18)] = 348,
  [SMALL_STATE(19)] = 367,
  [SMALL_STATE(20)] = 386,
  [SMALL_STATE(21)] = 405,
  [SMALL_STATE(22)] = 426,
  [SMALL_STATE(23)] = 445,
  [SMALL_STATE(24)] = 463,
  [SMALL_STATE(25)] = 485,
  [SMALL_STATE(26)] = 507,
  [SMALL_STATE(27)] = 519,
  [SMALL_STATE(28)] = 535,
  [SMALL_STATE(29)] = 551,
  [SMALL_STATE(30)] = 558,
  [SMALL_STATE(31)] = 567,
  [SMALL_STATE(32)] = 574,
  [SMALL_STATE(33)] = 587,
  [SMALL_STATE(34)] = 594,
  [SMALL_STATE(35)] = 598,
  [SMALL_STATE(36)] = 602,
  [SMALL_STATE(37)] = 606,
  [SMALL_STATE(38)] = 610,
  [SMALL_STATE(39)] = 614,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0, 0, 0),
  [5] = {.entry = {.count = 1, .reusable = false}}, SHIFT(36),
  [7] = {.entry = {.count = 1, .reusable = true}}, SHIFT(37),
  [9] = {.entry = {.count = 1, .reusable = false}}, SHIFT(31),
  [11] = {.entry = {.count = 1, .reusable = false}}, SHIFT(33),
  [13] = {.entry = {.count = 1, .reusable = true}}, SHIFT(4),
  [15] = {.entry = {.count = 1, .reusable = false}}, SHIFT(4),
  [17] = {.entry = {.count = 1, .reusable = true}}, SHIFT(38),
  [19] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0),
  [21] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(36),
  [24] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(37),
  [27] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(31),
  [30] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(33),
  [33] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(4),
  [36] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(4),
  [39] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(38),
  [42] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, 0, 0),
  [44] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_path, 1, 0, 0),
  [46] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_path, 1, 0, 0),
  [48] = {.entry = {.count = 2, .reusable = false}}, REDUCE(sym_path, 1, 0, 0), SHIFT(37),
  [51] = {.entry = {.count = 2, .reusable = false}}, REDUCE(sym_path, 1, 0, 0), SHIFT(5),
  [54] = {.entry = {.count = 2, .reusable = false}}, REDUCE(sym_path, 1, 0, 0), SHIFT(38),
  [57] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_path_repeat1, 2, 0, 0),
  [59] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_path_repeat1, 2, 0, 0),
  [61] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_path_repeat1, 2, 0, 0), SHIFT_REPEAT(37),
  [64] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_path_repeat1, 2, 0, 0), SHIFT_REPEAT(5),
  [67] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_path_repeat1, 2, 0, 0), SHIFT_REPEAT(38),
  [70] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_variable, 3, 0, 0),
  [72] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_variable, 3, 0, 0),
  [74] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_url_components, 1, 0, 0),
  [76] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_url_components, 1, 0, 0),
  [78] = {.entry = {.count = 1, .reusable = false}}, SHIFT(27),
  [80] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_key, 1, 0, 0),
  [82] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_key, 1, 0, 0),
  [84] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_query_string, 2, 0, 0),
  [86] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_query_string, 2, 0, 0),
  [88] = {.entry = {.count = 1, .reusable = false}}, SHIFT(28),
  [90] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_query_param, 1, 0, 0),
  [92] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_query_param, 1, 0, 0),
  [94] = {.entry = {.count = 1, .reusable = false}}, SHIFT(32),
  [96] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_query_string, 3, 0, 0),
  [98] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_query_string, 3, 0, 0),
  [100] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_query_string_repeat1, 2, 0, 0),
  [102] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_query_string_repeat1, 2, 0, 0),
  [104] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_query_string_repeat1, 2, 0, 0), SHIFT_REPEAT(28),
  [107] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_path_param, 2, 0, 0),
  [109] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_path_param, 2, 0, 0),
  [111] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_value, 1, 0, 0),
  [113] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_value, 1, 0, 0),
  [115] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_query_param, 3, 0, 0),
  [117] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_query_param, 3, 0, 0),
  [119] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_url_line, 2, 0, 0),
  [121] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_url_line, 2, 0, 0),
  [123] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_url_components, 2, 0, 0),
  [125] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_url_components, 2, 0, 0),
  [127] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_url_line, 1, 0, 0),
  [129] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_url_line, 1, 0, 0),
  [131] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 1, 0, 0),
  [133] = {.entry = {.count = 1, .reusable = true}}, SHIFT(23),
  [135] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 1, 0, 0),
  [137] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_url_line, 3, 0, 0),
  [139] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_url_line, 3, 0, 0),
  [141] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0),
  [143] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_protocol, 2, 0, 0),
  [145] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_protocol, 2, 0, 0),
  [147] = {.entry = {.count = 1, .reusable = true}}, SHIFT(8),
  [149] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_domain_and_port, 3, 0, 0),
  [151] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym_domain_and_port, 1, 0, 0), SHIFT(34),
  [154] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_domain_and_port, 1, 0, 0),
  [156] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_domain, 1, 0, 0),
  [158] = {.entry = {.count = 1, .reusable = false}}, SHIFT(38),
  [160] = {.entry = {.count = 1, .reusable = false}}, SHIFT(16),
  [162] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_hostname, 1, 0, 0),
  [164] = {.entry = {.count = 1, .reusable = true}}, SHIFT(29),
  [166] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [168] = {.entry = {.count = 1, .reusable = true}}, SHIFT(26),
  [170] = {.entry = {.count = 1, .reusable = true}}, SHIFT(13),
  [172] = {.entry = {.count = 1, .reusable = true}}, SHIFT(39),
  [174] = {.entry = {.count = 1, .reusable = true}}, SHIFT(6),
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
