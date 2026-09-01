import {
  Button,
  ErrorState,
  FormField,
  FormSection,
  Header,
  LoadingState,
  SpaceBetween,
} from "@araf/ui";
import { useState, useMemo, useRef, useCallback, useEffect } from "react";
import { validateFormData, type ValidationError } from "@araf/schema-runtime";
import { useResourceDescriptor } from "../hooks/useResourceDescriptor";
import { useCapabilities } from "../hooks/useCapabilities";
import { useCreateResource } from "../hooks/useCreateResource";
import type { ResourceDescriptor } from "../descriptor";

export interface ResourceCreatePageProps {
  readonly resourceType: string;
}

interface FieldSpec {
  readonly key: string;
  readonly label: string;
  readonly description: string | undefined;
  readonly helpText: string | undefined;
  readonly constraintText: string | undefined;
  readonly widget: "text" | "number" | "select" | "boolean";
  readonly required: boolean;
  readonly options: readonly string[] | undefined;
  readonly basic: boolean;
  readonly order: number;
}

interface FormFieldState {
  value: unknown;
  error: string | undefined;
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function getXaraf(property: Record<string, unknown>): Record<string, unknown> {
  const xAraf = property["x-araf"];
  return isPlainObject(xAraf) ? xAraf : {};
}

function buildFieldSpecs(descriptor: ResourceDescriptor): FieldSpec[] {
  const schema = descriptor.createSchema;
  if (!isPlainObject(schema)) return [];

  const properties = schema.properties;
  if (!isPlainObject(properties)) return [];

  const required = new Set(
    Array.isArray(schema.required)
      ? schema.required.filter((r): r is string => typeof r === "string")
      : [],
  );

  const specs: FieldSpec[] = [];
  for (const [key, rawProperty] of Object.entries(properties)) {
    if (!isPlainObject(rawProperty)) continue;

    const xAraf = getXaraf(rawProperty);
    const type = typeof rawProperty.type === "string" ? rawProperty.type : undefined;
    const enumValues = Array.isArray(rawProperty.enum)
      ? rawProperty.enum.filter((v): v is string => typeof v === "string")
      : undefined;
    const widgetOverride = typeof xAraf.widget === "string" ? xAraf.widget : undefined;

    let widget: FieldSpec["widget"] = "text";
    if (type === "boolean") {
      widget = "boolean";
    } else if (enumValues !== undefined || widgetOverride === "select") {
      widget = "select";
    } else if (type === "number" || type === "integer") {
      widget = "number";
    }

    const basic = xAraf.advanced !== true && xAraf.basic !== false;
    const order = typeof xAraf.order === "number" ? xAraf.order : Number.POSITIVE_INFINITY;

    specs.push({
      key,
      label:
        typeof xAraf.label === "string"
          ? xAraf.label
          : typeof rawProperty.title === "string"
            ? rawProperty.title
            : key,
      description:
        typeof rawProperty.description === "string" ? rawProperty.description : undefined,
      helpText: typeof xAraf.helpText === "string" ? xAraf.helpText : undefined,
      constraintText: buildConstraintText(rawProperty),
      widget,
      required: required.has(key),
      options: enumValues,
      basic,
      order,
    });
  }

  return specs.sort((a, b) => {
    if (a.order !== b.order) return a.order - b.order;
    return a.key.localeCompare(b.key);
  });
}

function buildConstraintText(property: Record<string, unknown>): string | undefined {
  const parts: string[] = [];
  if (typeof property.minLength === "number" && property.minLength > 0) {
    parts.push(`Minimum length: ${String(property.minLength)}`);
  }
  if (typeof property.minimum === "number") {
    parts.push(`Minimum: ${String(property.minimum)}`);
  }
  if (Array.isArray(property.enum)) {
    const values = property.enum.filter((v): v is string => typeof v === "string");
    if (values.length > 0) {
      parts.push(`Allowed values: ${values.join(", ")}`);
    }
  }
  return parts.length > 0 ? parts.join(". ") : undefined;
}

function getInitialValue(spec: FieldSpec): unknown {
  if (spec.widget === "boolean") return false;
  return "";
}

function buildFormState(specs: FieldSpec[]): Record<string, FormFieldState> {
  const state: Record<string, FormFieldState> = {};
  for (const spec of specs) {
    state[spec.key] = { value: getInitialValue(spec), error: undefined };
  }
  return state;
}

function buildPayload(
  formState: Record<string, FormFieldState>,
  specs: FieldSpec[],
): Record<string, unknown> {
  const payload: Record<string, unknown> = {};
  const specMap = new Map(specs.map((spec) => [spec.key, spec]));
  for (const [key, field] of Object.entries(formState)) {
    const spec = specMap.get(key);
    if (!spec?.required && field.value === "") {
      continue;
    }
    payload[key] = field.value;
  }
  return payload;
}

function collectErrors(errors: readonly ValidationError[]): Record<string, string> {
  const map: Record<string, string> = {};
  for (const error of errors) {
    const missingProperty =
      error.keyword === "required" && typeof error.params?.missingProperty === "string"
        ? error.params.missingProperty
        : undefined;
    const key = missingProperty ?? error.instancePath.replace(/^\//u, "");
    if (key && !(key in map)) {
      map[key] = error.message;
    }
  }
  return map;
}

function hasCapability(
  capabilities: readonly { readonly resourceType: string; readonly action: string }[],
  required: { readonly resourceType: string; readonly action: string },
): boolean {
  return capabilities.some(
    (c) => c.resourceType === required.resourceType && c.action === required.action,
  );
}

export function ResourceCreatePage({ resourceType }: ResourceCreatePageProps) {
  const {
    descriptor,
    loading: descriptorLoading,
    error: descriptorError,
  } = useResourceDescriptor(resourceType);
  const { capabilities, loading: capabilitiesLoading } = useCapabilities();
  const {
    create,
    loading: createLoading,
    error: createError,
    operation,
  } = useCreateResource(resourceType);
  const [advancedExpanded, setAdvancedExpanded] = useState(false);
  const [formState, setFormState] = useState<Record<string, FormFieldState>>({});
  const [submitError, setSubmitError] = useState<string | undefined>(undefined);
  const fieldRefs = useRef<Record<string, HTMLElement | null>>({});

  const specs = useMemo(() => (descriptor ? buildFieldSpecs(descriptor) : []), [descriptor]);

  useEffect(() => {
    setFormState(buildFormState(specs));
    setSubmitError(undefined);
  }, [specs]);

  const basicSpecs = useMemo(() => specs.filter((s) => s.basic), [specs]);
  const advancedSpecs = useMemo(() => specs.filter((s) => !s.basic), [specs]);

  const setFieldValue = useCallback((key: string, value: unknown) => {
    setFormState((prev) => ({
      ...prev,
      [key]: { ...prev[key], value, error: undefined },
    }));
    setSubmitError(undefined);
  }, []);

  const setFieldErrors = useCallback((errors: Record<string, string | undefined>) => {
    setFormState((prev) => {
      const next: Record<string, FormFieldState> = { ...prev };
      for (const [key, error] of Object.entries(errors)) {
        if (key in next) {
          const field = next[key];
          if (field) {
            next[key] = { ...field, error };
          }
        }
      }
      return next;
    });
  }, []);

  const handleSubmit = async (event: React.SyntheticEvent<HTMLFormElement>): Promise<void> => {
    event.preventDefault();
    if (!descriptor) return;

    setFieldErrors(Object.fromEntries(Object.keys(formState).map((key) => [key, undefined])));
    setSubmitError(undefined);

    const payload = buildPayload(formState, specs);
    const errors = validateFormData(descriptor.createSchema, payload);

    if (errors.length > 0) {
      const mapped = collectErrors(errors);
      setFieldErrors(mapped);
      setSubmitError("Please correct the errors below.");
      const firstKey = Object.keys(mapped)[0];
      if (firstKey) {
        fieldRefs.current[firstKey]?.focus();
      }
      return;
    }

    await create(payload);
  };

  if (descriptorLoading || capabilitiesLoading) {
    return <LoadingState message="Loading create form..." />;
  }

  if (descriptorError) {
    return (
      <ErrorState
        title="Could not load resource descriptor"
        message={
          descriptorError instanceof Error ? descriptorError.message : String(descriptorError)
        }
      />
    );
  }

  if (!descriptor) {
    return (
      <ErrorState
        title="Resource type not found"
        message={`No descriptor found for ${resourceType}.`}
      />
    );
  }

  if (!descriptor.createSchema) {
    return (
      <ErrorState
        title="Create not available"
        message={`${descriptor.name} does not support creation through the console.`}
      />
    );
  }

  if (!hasCapability(capabilities, descriptor.createCapability)) {
    return (
      <ErrorState
        title="Create not available"
        message={`You do not have the ${descriptor.createCapability.resourceType}/${descriptor.createCapability.action} capability required to create ${descriptor.pluralName}.`}
      />
    );
  }

  if (operation) {
    return (
      <section aria-label={`${descriptor.name} created`}>
        <Header variant="h1" headingLevel="h1">
          {descriptor.name} creation submitted
        </Header>
        <p>
          Operation <strong>{operation.id}</strong> is <strong>{operation.state}</strong>.
        </p>
        <p>Correlation ID: {operation.correlationId}</p>
        <Button href={`/resources/${encodeURIComponent(resourceType)}`} variant="primary">
          View {descriptor.pluralName}
        </Button>
      </section>
    );
  }

  return (
    <section aria-label={`Create ${descriptor.name}`}>
      <Header variant="h1" headingLevel="h1">
        Create {descriptor.name}
      </Header>

      {submitError ? (
        <div role="alert" style={{ color: "red", marginBottom: "1rem" }}>
          {submitError}
        </div>
      ) : null}

      {createError ? (
        <div role="alert" style={{ color: "red", marginBottom: "1rem" }}>
          {createError instanceof Error ? createError.message : String(createError)}
        </div>
      ) : null}

      <form
        onSubmit={(event) => {
          void handleSubmit(event);
        }}
        noValidate
      >
        <SpaceBetween size="l" direction="vertical">
          <FormSection title="Basic configuration">
            <SpaceBetween size="l" direction="vertical">
              {basicSpecs.map((spec) => (
                <FieldControl
                  key={spec.key}
                  spec={spec}
                  state={formState[spec.key] ?? { value: "", error: undefined }}
                  onChange={(value) => {
                    setFieldValue(spec.key, value);
                  }}
                  inputRef={(el) => {
                    fieldRefs.current[spec.key] = el;
                  }}
                />
              ))}
            </SpaceBetween>
          </FormSection>

          {advancedSpecs.length > 0 ? (
            <FormSection title="Advanced" description="Optional advanced configuration.">
              <SpaceBetween size="l" direction="vertical">
                {advancedExpanded
                  ? advancedSpecs.map((spec) => (
                      <FieldControl
                        key={spec.key}
                        spec={spec}
                        state={formState[spec.key] ?? { value: "", error: undefined }}
                        onChange={(value) => {
                          setFieldValue(spec.key, value);
                        }}
                        inputRef={(el) => {
                          fieldRefs.current[spec.key] = el;
                        }}
                      />
                    ))
                  : null}
                <Button
                  formAction="none"
                  variant="link"
                  onClick={() => {
                    setAdvancedExpanded((prev) => !prev);
                  }}
                >
                  {advancedExpanded ? "Hide advanced" : "Show advanced"}
                </Button>
              </SpaceBetween>
            </FormSection>
          ) : null}

          <Button variant="primary" loading={createLoading}>
            Create {descriptor.name}
          </Button>
        </SpaceBetween>
      </form>
    </section>
  );
}

interface FieldControlProps {
  readonly spec: FieldSpec;
  readonly state: FormFieldState;
  readonly onChange: (value: unknown) => void;
  readonly inputRef?: (el: HTMLElement | null) => void;
}

function FieldControl({ spec, state, onChange, inputRef }: FieldControlProps) {
  const id = `field-${spec.key}`;
  const description = spec.description ?? spec.helpText;

  const handleTextChange = (
    event: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>,
  ): void => {
    onChange(event.target.value);
  };

  const handleNumberChange = (event: React.ChangeEvent<HTMLInputElement>): void => {
    const value = event.target.value;
    if (value === "") {
      onChange("");
      return;
    }
    const parsed = Number.parseFloat(value);
    onChange(Number.isNaN(parsed) ? value : parsed);
  };

  const handleBooleanChange = (event: React.ChangeEvent<HTMLInputElement>): void => {
    onChange(event.target.checked);
  };

  const commonProps = {
    id,
    required: spec.required,
    "aria-required": spec.required,
  };

  const setRef = (el: HTMLInputElement | HTMLSelectElement | null): void => {
    inputRef?.(el);
  };

  const numberValue =
    typeof state.value === "number"
      ? state.value
      : typeof state.value === "string"
        ? state.value
        : "";

  return (
    <FormField
      id={id}
      label={spec.label}
      description={description}
      constraintText={spec.constraintText}
      errorText={state.error}
    >
      {spec.widget === "boolean" ? (
        <input
          {...commonProps}
          ref={setRef}
          type="checkbox"
          checked={Boolean(state.value)}
          onChange={handleBooleanChange}
        />
      ) : spec.widget === "select" ? (
        <select
          {...commonProps}
          ref={setRef}
          value={typeof state.value === "string" ? state.value : ""}
          onChange={handleTextChange}
        >
          <option value="">{spec.required ? "Select..." : "None"}</option>
          {spec.options?.map((option) => (
            <option key={option} value={option}>
              {option}
            </option>
          ))}
        </select>
      ) : spec.widget === "number" ? (
        <input
          {...commonProps}
          ref={setRef}
          type="number"
          value={numberValue}
          onChange={handleNumberChange}
        />
      ) : (
        <input
          {...commonProps}
          ref={setRef}
          type="text"
          value={typeof state.value === "string" ? state.value : ""}
          onChange={handleTextChange}
        />
      )}
    </FormField>
  );
}
