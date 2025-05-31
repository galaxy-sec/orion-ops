{{/* 定义模板 */}}
{{- define "greeting" -}}
   Hello, {{ .Name }}!
{{- end -}}

{{/* 调用模板 */}}
{{ template "greeting" . }}
