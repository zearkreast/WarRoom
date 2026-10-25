import { describe, expect, it } from "vitest";
import { notEnglish } from "./language.ts";

const dispatch = (message: string) => [{ id: 1, published: "2026-10-25T08:01:30Z", type: 0, message }];

describe("language check", () => {
  it("passes English, planet names with accents and bare timestamps", () => {
    expect(
      notEnglish(dispatch("<i=3>NEW ORDERS</i> Helldivers, hold the line on <i=1>Pöpli IX</i> for Liberty.")),
    ).toBe(false);
    expect(notEnglish({ started: "2024-01-23T20:05:13Z", factions: ["Humans", "Terminids"] })).toBe(false);
  });

  it.each([
    ["Spanish", "Los Helldivers deben defender la colonia de los ataques del enemigo para la Libertad."],
    ["German", "Die Helldivers müssen die Kolonie gegen den Feind verteidigen und nicht weichen."],
    ["French", "Les Helldivers doivent défendre la colonie contre les attaques de l'ennemi pour la Liberté."],
    ["Italian", "Gli Helldivers devono difendere la colonia dagli attacchi del nemico per la Libertà."],
    ["Portuguese", "Os Helldivers devem defender a colônia dos ataques do inimigo pela Liberdade."],
    ["Polish", "Helldiverzy muszą bronić kolonii przed atakami wroga, aby zachować Wolność."],
    ["Russian", "Адские десантники должны защитить колонию от врага."],
    ["Chinese", "绝地潜兵必须保卫殖民地，抵御敌人的进攻。"],
    ["Japanese", "ヘルダイバーは敵の攻撃から植民地を守らなければならない。"],
    ["Korean", "헬다이버는 적의 공격으로부터 식민지를 지켜야 한다."],
  ])("catches %s", (_, text) => {
    expect(notEnglish(dispatch(text))).toBe(true);
  });
});
