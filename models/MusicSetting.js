const mongoose = require('mongoose');

const MusicSettingSchema = new mongoose.Schema({
    guildId: { type: String, required: true, unique: true },
    channelId: { type: String, required: true },
    messageId: { type: String, required: true },
    systemEnabled: { type: Boolean, default: true }
});

module.exports = mongoose.model('MusicSetting', MusicSettingSchema);