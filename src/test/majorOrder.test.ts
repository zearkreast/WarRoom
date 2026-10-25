import { describe, expect, it } from "vitest";
import { orderTaskRates, orderVerdict, taskOutlook, type TaskOutlook, type TaskView } from "../domain/majorOrder";

const H = 3_600_000;

describe("Major Order outlook", () => {
  it("measures each task's count per hour from the order's readings", () => {
    const samples = [0, 1, 2].map((h) => ({ id: 7, at: h * H, progress: [h * 1000, 50] }));
    samples.push({ id: 8, at: 0, progress: [999_999, 0] });
    expect(orderTaskRates(samples, 7, 2)).toEqual([1000, 0]);
  });

  it("waits for half an hour of readings before trusting a rate", () => {
    const samples = [0, 10, 20].map((m) => ({ id: 7, at: m * 60_000, progress: [m] }));
    expect(orderTaskRates(samples, 7, 1)).toEqual([null]);
    expect(orderTaskRates(undefined, 7, 2)).toEqual([null, null]);
  });

  it("wins when the pace reaches 100% before the deadline, with a time to completion", () => {
    expect(taskOutlook(50, 10, 0, 10 * H)).toEqual({ outlook: "on-track", end: 100, eta: 5 * H, rate: 10 });
    expect(taskOutlook(50, 2, 0, 10 * H)).toEqual({ outlook: "behind", end: 70, eta: null, rate: 2 });
    expect(taskOutlook(50, null, 0, 10 * H).outlook).toBe("measuring");
    expect(taskOutlook(100, 3, 0, 10 * H).outlook).toBe("done");
  });

  it("calls the order lost if any objective falls short", () => {
    const t = (label: string): TaskView => ({ kind: "count", label, current: 0, goal: 1, done: false });
    const o = (outlook: TaskOutlook["outlook"]): TaskOutlook => ({ outlook, end: null, eta: null, rate: null });
    expect(orderVerdict([t("A"), t("B")], [o("on-track"), o("behind")])?.text).toMatch(/^Losing: B/);
    expect(orderVerdict([t("A"), t("B")], [o("on-track"), o("done")])?.text).toMatch(/^Winning/);
    expect(orderVerdict([t("A"), t("B")], [o("on-track"), o("measuring")])?.text).toMatch(/^Measuring/);
  });
});
