{{/*
Expand the name of the chart.
*/}}
{{- define "f1r3fly.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "f1r3fly.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "f1r3fly.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "f1r3fly.labels" -}}
helm.sh/chart: {{ include "f1r3fly.chart" . }}
{{ include "f1r3fly.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "f1r3fly.selectorLabels" -}}
app.kubernetes.io/name: {{ include "f1r3fly.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "f1r3fly.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "f1r3fly.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Creates a list of public keys and does validation
*/}}
{{- define "f1r3fly.publicKeys" -}}
{{- if gt (int .Values.replicaCount) (len .Values.nodeKeys) }}
{{ fail ( printf "Can't deploy %d replicas: %d public/private keys defined only. Define more node keys" (int .Values.replicaCount) (len .Values.nodeKeys) ) }}
{{- end }}
{{- range $keys := mustSlice .Values.nodeKeys 0 .Values.replicaCount }}
{{ $keys.publicKey }}
{{- end }}
{{- end }}
