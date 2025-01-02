export { referrer as referrerExternalStatic } from "./resources/referrer-checker.py?name=external-static"
export const { referrer: referrerExternalDynamic } = await import("./resources/referrer-checker.py?name=external-dynamic");
