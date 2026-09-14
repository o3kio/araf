{{- define "araf.image" -}}
{{ .Values.image.repository }}@{{ required "image.digest is required" .Values.image.digest }}
{{- end }}
{{- define "araf.frontendImage" -}}
{{ .Values.frontendImage.repository }}@{{ required "frontendImage.digest is required" .Values.frontendImage.digest }}
{{- end }}
