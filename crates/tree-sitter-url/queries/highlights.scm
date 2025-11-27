; URL Variables: {{variableName}}
(variable_name) @variable

; Path Parameters: :path-param
(path_param) @variable.special

; Protocol
(protocol) @keyword

; Domain names
(domain) @type
(hostname) @type
(port) @number

; Query string and parameters
(query_string) @keyword
(key) @property
(value) @string

; Path segments
(path_segment) @string

; Delimiters for visual separation
(variable_delim_start) @punctuation.bracket
(variable_delim_end) @punctuation.bracket
"/" @punctuation.delimiter
"?" @punctuation.delimiter
"=" @operator
"&" @punctuation.delimiter

; URL line as the main structure
(url_line) @string.special