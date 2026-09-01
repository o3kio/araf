import { getScopeFromUrl, syncScopeToUrl, mergeScopeWithUrl } from "./url";

describe("getScopeFromUrl", () => {
  afterEach(() => {
    window.history.replaceState(null, "", "/");
  });

  it("reads project and region from query params", () => {
    window.history.replaceState(null, "", "/?project=p1&region=eu-west");
    expect(getScopeFromUrl()).toEqual({ projectId: "p1", regionId: "eu-west" });
  });

  it("treats region=global as the global scope", () => {
    window.history.replaceState(null, "", "/?region=global");
    expect(getScopeFromUrl()).toEqual({ regionId: "global" });
  });

  it("reads organization id", () => {
    window.history.replaceState(null, "", "/?org=acme&project=p1");
    expect(getScopeFromUrl()).toEqual({ organizationId: "acme", projectId: "p1" });
  });

  it("returns empty object when no scope params exist", () => {
    window.history.replaceState(null, "", "/?other=value");
    expect(getScopeFromUrl()).toEqual({});
  });
});

describe("syncScopeToUrl", () => {
  afterEach(() => {
    window.history.replaceState(null, "", "/");
  });

  it("writes scope params", () => {
    syncScopeToUrl({ projectId: "p1", regionId: "eu-west" });
    expect(window.location.search).toBe("?project=p1&region=eu-west");
  });

  it("writes global region", () => {
    syncScopeToUrl({ regionId: "global" });
    expect(window.location.search).toBe("?region=global");
  });

  it("preserves non-scope params", () => {
    window.history.replaceState(null, "", "/?other=value");
    syncScopeToUrl({ projectId: "p1" });
    expect(window.location.search).toContain("other=value");
    expect(window.location.search).toContain("project=p1");
  });

  it("removes cleared params", () => {
    window.history.replaceState(null, "", "/?project=p1&region=eu-west");
    syncScopeToUrl({});
    expect(window.location.search).toBe("");
  });
});

describe("mergeScopeWithUrl", () => {
  it("url scope overrides defaults", () => {
    expect(
      mergeScopeWithUrl({ projectId: "default", regionId: "global" }, { projectId: "url-project" }),
    ).toEqual({ projectId: "url-project", regionId: "global" });
  });
});
