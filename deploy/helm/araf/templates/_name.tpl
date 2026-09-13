{{- define "araf.fullname" -}}{{ .Release.Name }}-{{ .Chart.Name }}{{- end }}
