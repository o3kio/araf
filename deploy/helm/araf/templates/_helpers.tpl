{{- define "araf.image" -}}
{{- if not (regexMatch "^sha256:[a-f0-9]{64}$" .Values.image.digest) }}{{ fail "image.digest must be a sha256 digest" }}{{ end -}}
{{ .Values.image.repository }}@{{ required "image.digest is required" .Values.image.digest }}
{{- end }}
{{- define "araf.frontendImage" -}}
{{- $image := index .Values.frontendImage .surface -}}
{{- if not (regexMatch "^sha256:[a-f0-9]{64}$" $image.digest) }}{{ fail "frontend digest must be a sha256 digest" }}{{ end -}}
{{ $image.repository }}@{{ required (printf "frontendImage.%s.digest is required" .surface) $image.digest }}
{{- end }}
