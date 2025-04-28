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

const { Events } = require('discord.js');
const { useMainPlayer, QueryType, useQueue } = require('discord-player');
const MusicSetting = require('../../models/MusicSetting');

module.exports = {
  name: Events.MessageCreate,
  async execute(message) {

    if (!message.guild || message.author.bot) return;

    const setting = await MusicSetting.findOne({ guildId: message.guildId });
    if (!setting?.systemEnabled || message.channel.id !== setting.channelId) return;

    const memberVoice = message.member.voice.channel;
    if (!memberVoice) {
      await message.delete().catch(() => {});
      return message.reply({ content: ':x: | Lütfen önce ses kanalına katılın.' });
    }

    const query = message.content.trim();
    if (!query) return;
    await message.delete().catch(() => {});
    await message.channel.sendTyping();

    const player = useMainPlayer();
    const searchResult = await player.search(query, {
      requestedBy: message.member,
      searchEngine: QueryType.AUTO
    });

    if (!searchResult || !searchResult.tracks.length) {
      return message.channel.send(':x: | Şarkı bulunamadı.');
    }

    let queue = useQueue(message.guild.id);
    if (!queue) {
      queue = player.nodes.create(message.guild, {
        metadata: { channel: message.channel, requestedBy: message.member }
      });
    }

    try {
      if (!queue.connection) await queue.connect(memberVoice);
    } catch (err) {
      console.error('Ses kanalına bağlanırken hata:', err);
      return message.channel.send(':x: | Ses kanalına bağlanılamadı.');
    }

    if (searchResult.playlist) {
      queue.addTrack(searchResult.tracks);
      if (!queue.node.isPlaying()) {
        await queue.node.play();
      }
      await message.channel.send(`🎶 **${searchResult.tracks.length}** parçalık playlist kuyruğa eklendi!`);
    } 
    
    else {
      const track = searchResult.tracks[0];
      queue.addTrack(track);
      if (!queue.node.isPlaying()) {
        await queue.node.play();
      }
      await message.channel.send(`🎶 **${track.title}** kuyruğa eklendi!`);
    }

  }
};
