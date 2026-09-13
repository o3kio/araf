{{- define "araf.image" -}}
{{ .Values.image.repository }}@{{ required "image.digest is required" .Values.image.digest }}
{{- end }}
