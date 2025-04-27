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

// İstemci (Client) yapılandırması
const client = new Client({
  intents: [
    GatewayIntentBits.Guilds,
    GatewayIntentBits.GuildVoiceStates,
  ],
  partials: [Partials.Channel]
});

// Global ayarlar ve koleksiyonların tanımlanması
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
    YoutubeiExtractor
  ],
  blockExtractors: [
    YoutubeiExtractor
  ],

});

async function init() {
  await player.extractors.register(SpotifyExtractor, {});
  await player.extractors.register(SoundcloudExtractor, {});
} init();


client.on(Events.ClientReady, () => {
  require('./utils/command.js')(client);
});

fs.readdirSync('./handlers').forEach(handler => {
  require(`./handlers/${handler}`)(client);
});

client.login(process.env.TOKEN);

player.events.on('playerStart', (queue, track) => {

  const buttons = new ActionRowBuilder().addComponents(
    new ButtonBuilder()
      .setCustomId('pause')
      .setLabel('Durdur')
      .setStyle(ButtonStyle.Primary),
    new ButtonBuilder()
      .setCustomId('resume')
      .setLabel('Devam')
      .setStyle(ButtonStyle.Success),
    new ButtonBuilder()
      .setCustomId('stop')
      .setLabel('Bitir')
      .setStyle(ButtonStyle.Danger),
    new ButtonBuilder()
      .setCustomId('skip')
      .setLabel('Atla')
      .setStyle(ButtonStyle.Secondary),
    new ButtonBuilder()
      .setCustomId('volume')
      .setLabel('Ses Ayarla')
      .setStyle(ButtonStyle.Secondary)
  );

  queue.metadata.channel.send({ content: `🎶 **${track.title}** oynamaya başladı!`, components: [buttons] });

});

// Kanal boşsa otomatik ayrılma
player.events.on('emptyChannel', (queue) => {
  queue.metadata.channel.send('❌ | Kanalda kimse kalmadığı için ayrılıyorum.');
});

// Kuyruk sona erdiğinde, 1 dakika içinde yeni parça eklenmezse kanaldan çıkma
player.events.on('emptyQueue', (queue) => {
  setTimeout(() => {
    if (!queue.isPlaying) {
      queue.metadata.channel.send('⏳ | 1 dakika boyunca oynatma yok, kanaldan ayrılıyorum.');
      if (queue.connection) queue.connection.disconnect();
    }
  }, 60000);
});

player.events.on('error', (queue, error) => {
  console.log(`General player error event: ${error.message}`);
  console.log(error);
});

player.events.on('playerError', (queue, error) => {
  console.log(`Player error event: ${error.message}`);
  console.log(error);
});

module.exports = client;
