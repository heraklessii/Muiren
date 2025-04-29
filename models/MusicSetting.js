const mongoose = require('mongoose');

const MusicSettingSchema = new mongoose.Schema({
    guildId: { type: String, required: true, unique: true },
    channelId: { type: String, required: false, default: null },
    messageId: { type: String, required: false, default: null },
    djRoleID: { type: String, required: false, default: null },
    systemEnabled: { type: Boolean, default: false }
});

module.exports = mongoose.model('MusicSetting', MusicSettingSchema);