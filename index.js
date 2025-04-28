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
const { YoutubeiExtractor } = require("discord-player-youtubei");
const { SoundcloudExtractor } = require("discord-player-soundcloud");
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
client.color = "#2b2d32";
client.red = "#FC6161";
client.green = "#73D673";
client.white = "#FFFFFF";
client.black = "#000001";
client.blue = "#449afe";
client.yellow = "#FFD700";

const player = new Player(client, {
  blockStreamFrom: [
    SpotifyExtractor
  ],
  blockExtractors: [
    SoundcloudExtractor
  ],
});

(async () => {
  await player.extractors.register(SpotifyExtractor);
  await player.extractors.register(YoutubeiExtractor);
})();

require("./utils/player.js")(client);

client.on(Events.ClientReady, () => {
  require('./utils/command.js')(client);
});

fs.readdirSync('./handlers').forEach(handler => {
  require(`./handlers/${handler}`)(client);
});

client.login(process.env.TOKEN);

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
