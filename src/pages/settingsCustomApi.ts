export interface CustomApiProvider {
  id: string;
  name: string;
  base_url: string;
  model: string;
  api_key: string | null;
}

export type CustomApiProviderKey = `custom:${string}`;

export function createCustomApiProvider(): CustomApiProvider {
  const id =
    typeof crypto !== "undefined" && "randomUUID" in crypto
      ? crypto.randomUUID()
      : `custom-${Date.now()}`;

  return {
    id,
    name: "",
    base_url: "",
    model: "",
    api_key: null,
  };
}

export function customApiProviderKey(provider: Pick<CustomApiProvider, "id">): CustomApiProviderKey {
  return `custom:${provider.id}`;
}

export function normalizeCustomApiProvider(provider: CustomApiProvider): CustomApiProvider {
  return {
    id: provider.id.trim(),
    name: provider.name.trim(),
    base_url: provider.base_url.trim().replace(/\/+$/, ""),
    model: provider.model.trim(),
    api_key: provider.api_key?.trim() || null,
  };
}

export function isCustomApiProviderKey(providerKey: string): providerKey is CustomApiProviderKey {
  return providerKey.startsWith("custom:");
}

export function customApiProviderIdFromKey(providerKey: string): string | null {
  return isCustomApiProviderKey(providerKey) ? providerKey.slice("custom:".length) : null;
}
