// Muiren Bot - Discord Bot Project
// Copyright (C) 2025  Muiren Bot Contributors
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

require('dotenv').config();
const { Client, GatewayIntentBits, Partials, Collection, Events, ActionRowBuilder, EmbedBuilder, ButtonBuilder, ButtonStyle } = require('discord.js');
const { Player } = require('discord-player');
const { SpotifyExtractor } = require("discord-player-spotify");
const { SoundcloudExtractor } = require("discord-player-soundcloud");
const { YoutubeiExtractor } = require("discord-player-youtubei");
const fs = require('node:fs');

const client = new Client({
  intents: [
    GatewayIntentBits.Guilds,
    GatewayIntentBits.GuildVoiceStates,
    GatewayIntentBits.GuildMessages,
    GatewayIntentBits.MessageContent
  ],
  partials: [Partials.Channel, Partials.Message]
});

global.client = client;
client.commands = new Collection();
client.cooldowns = new Collection();

// Genel embed renkleri.
client.color = "#cf93ff";
client.red = "#FC6161";
client.green = "#73D673";

const player = new Player(client, {
  // Eğer Soundcloud üzerinden oynatmamasını isterseniz yorum satırlarını kaldırabilirsiniz.
  // UNUTMAYIN! Buraya asla Youtube eklememelisiniz yoksa şarkı oynatamazsınız.
  // Tek başına Soundcloud sadece o platform üzerinden stream yapabilmenizi sağlar.
  blockExtractors: [/*SoundcloudExtractor.identifier*/],
  blockStreamFrom: [/*SoundcloudExtractor.identifier*/],
  // Buradaki ayarlara dokunmanızı önermem, şu anki hâli ile ses kalitesi oldukça iyi.
  smoothVolume: true,
  ytdlOptions: {
    filter: 'audioonly',
    quality: 'highestaudio',
    highWaterMark: 1 << 30,
    dlChunkSize: 0,
  }
});

// Bu kısımdan register edilip edilmemesini ayarlayabilirsiniz.
// Spotify bilgilerinizi ayarlarsanız daha iyi sonuçlar alabilirsiniz.
// https://developer.spotify.com
// UNUTMAYIN! Eğer Youtube veya Soundcloud koymazsanız stream yapamazsınız.
// Direkt olarak Spotify üzerinden maalesef yayın yapılamıyor.
// https://www.npmjs.com/package/discord-player-spotify
// Sadece metada olarak kullanabilirsiniz.
(async () => {
  await player.extractors.register(SpotifyExtractor, {
    clientId: process.env.SPOTIFY_ID ?? null,
    clientSecret: process.env.SPOTIFY_SECRET ?? null
  });
  await player.extractors.register(YoutubeiExtractor);
  await player.extractors.register(SoundcloudExtractor);
})();

require("./utils/player.js")(client);

client.on(Events.ClientReady, () => {
  require('./utils/command.js')(client);
});

fs.readdirSync('./handlers').forEach(handler => {
  require(`./handlers/${handler}`)(client);
});

client.login(process.env.TOKEN);

// Buradaki uyarılar önemli şeyler değil ancak yer kaplıyor.
// Bu yüzden gizlemeyi tercih ettim. Dilerseniz buradaki satırları silebilirsiniz.
const originalWarn = console.warn;
console.warn = (...args) => {
  // Yalnızca “[YOUTUBEJS]” ile başlayan uyarıları yoksay
  if (typeof args[0] === 'string' && args[0].startsWith('[YOUTUBEJS]')) {
    return;
  }
  originalWarn.apply(console, args);
};

const originalDebug = console.debug;
console.debug = (...args) => {
  if (typeof args[0] === 'string' && args[0].startsWith('[YOUTUBEJS]')) {
    return;
  }
  originalDebug.apply(console, args);
};

module.exports = client;
