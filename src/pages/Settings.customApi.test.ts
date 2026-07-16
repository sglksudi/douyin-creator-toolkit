import {
  createCustomApiProvider,
  customApiProviderKey,
  normalizeCustomApiProvider,
  type CustomApiProvider,
} from "./settingsCustomApi";

const draft = createCustomApiProvider();
draft.name = "Silicon Flow";
draft.base_url = "https://api.siliconflow.cn/v1";
draft.model = "Qwen/Qwen3-235B-A22B";
draft.api_key = "sk-test";

const selectedKey: `custom:${string}` = customApiProviderKey(draft);

const normalized: CustomApiProvider = normalizeCustomApiProvider({
  id: " silicon-flow ",
  name: " Silicon Flow ",
  base_url: " https://api.siliconflow.cn/v1/ ",
  model: " Qwen/Qwen3-235B-A22B ",
  api_key: " ",
});

void selectedKey;
void normalized;
