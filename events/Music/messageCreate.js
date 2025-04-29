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

const { Events, PermissionsBitField } = require('discord.js');
const { useMainPlayer, QueryType, useQueue } = require('discord-player');
const { UpdateQueueMsg } = require("../../utils/musicUpdater");
const MusicSetting = require('../../models/MusicSetting');
module.exports = {
  name: Events.MessageCreate,
  async execute(message) {

    if (!message.guild || message.author.bot) return;

    const setting = await MusicSetting.findOne({ guildId: message.guildId });
    if (!setting?.systemEnabled || message.channel.id !== setting.channelId) return;

    // ---- İzin Kontrolleri ---- //
    const memberVoice = await checks(message);
    if (!memberVoice) return;

    const query = message.content.trim();
    if (!query) return;
    await message.delete().catch(() => { });
    await message.channel.sendTyping();

    const player = useMainPlayer();
    let searchResult;
    try {
      searchResult = await player.search(query, {
        requestedBy: message.member,
        searchEngine: QueryType.AUTO
      });
      if (!searchResult || !searchResult.tracks.length) {
        throw new Error('NO_TRACKS');
      }
    } catch (err) {
      return message.channel.send('❌ | Şarkı bulunamadı veya akışa ulaşılamadı.')
        .then(sent => setTimeout(() => sent.delete().catch(() => { }), 3000));
    }

    let queue = useQueue(message.guild.id);
    if (!queue) {
      queue = player.nodes.create(message.guild, {
        metadata: { channel: message.channel, requestedBy: message.member },
        leaveOnEnd: true,
        leaveOnEndCooldown: 60000
      });
    }

    try {
      if (!queue.connection) await queue.connect(memberVoice);
    } catch (err) {
      queue.delete();
      return message.channel.send('❌ | Ses kanalına bağlanılamadı.')
        .then(sent => setTimeout(() => sent.delete().catch(() => { }), 3000));
    }

    // Playlist
    if (searchResult.playlist) {
      try {
        queue.addTrack(searchResult.tracks);
        if (!queue.node.isPlaying()) await queue.node.play();
        await message.channel.send(`🎶 | **${searchResult.tracks.length}** parçalık playlist kuyruğa eklendi!`)
          .then(sent => setTimeout(() => sent.delete().catch(() => { }), 3000));
      } catch (err) {
        queue.delete();
        await message.channel.send('❌ | Playlist oynatılamadı.')
          .then(sent => setTimeout(() => sent.delete().catch(() => { }), 3000));
      }
      return;
    }

    // Tek parça
    const track = searchResult.tracks[0];
    try {
      queue.addTrack(track);
      if (!queue.node.isPlaying()) await queue.node.play();
      await message.channel.send(`🎶 | **${track.title}** kuyruğa eklendi!`)
        .then(sent => setTimeout(() => sent.delete().catch(() => { }), 3000));
    } catch (err) {
      queue.delete();
      return message.channel.send('❌ | Şarkı oynatılamıyor.')
        .then(sent => setTimeout(() => sent.delete().catch(() => { }), 3000));
    }

  }
};

async function checks(message) {

  const voiceChannel = message.member.voice.channel;
  if (!voiceChannel) {
    await message.channel.send({ content: `❌ | ${message.author} Ses kanalında değilsiniz.` }).then(sent => {
      setTimeout(() => sent.delete().catch(() => { }), 3000);
    });
    return null;
  }

  const permissions = voiceChannel.permissionsFor(message.guild.members.me);
  if (!permissions.has(PermissionsBitField.Flags.Connect)) {
    await message.channel.send({ content: `❌ | ${message.author} Kanala bağlanma iznim yok.` }).then(sent => {
      setTimeout(() => sent.delete().catch(() => { }), 3000);
    });
    return null;
  }

  const botVoice = message.guild.members.me.voice.channel;
  if (botVoice && botVoice.id !== voiceChannel.id) {
    await message.channel.send({ content: `❌ | ${message.author} Başka bir ses kanalındayım. ${botVoice}` }).then(sent => {
      setTimeout(() => sent.delete().catch(() => { }), 3000);
    });
    return null;
  }

  if (!permissions.has(PermissionsBitField.Flags.Speak)) {
    await message.channel.send({ content: `❌ | ${message.author} Konuşma iznim olmadığı için şarkı oynatamıyorum.` }).then(sent => {
      setTimeout(() => sent.delete().catch(() => { }), 3000);
    });
    return null;
  }

  return voiceChannel;
}