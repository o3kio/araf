{{- define "araf.image" -}}
{{ .Values.image.repository }}@{{ required "image.digest is required" .Values.image.digest }}
{{- end }}
{{- define "araf.frontendImage" -}}
{{- $image := index .Values.frontendImage .surface -}}
{{ $image.repository }}@{{ required (printf "frontendImage.%s.digest is required" .surface) $image.digest }}
{{- end }}
