extends Label3D

func popup(value: int):
	var dup = self.duplicate()
	self.add_sibling(dup)
	dup._popup_internal(value)

func _popup_internal(value: int):
	self.visible = true
	
	self.text = str(value)
	var t_end = 1
	
	# animate offset and not position due to billboarding shenanigans.

	var drift_x = 0 #randf() * 25 * (-1 if randf() < 0.5 else 1)
	self.create_tween().tween_property(self, "offset:x", drift_x, t_end).set_trans(Tween.TRANS_LINEAR)

	var peak_y = 15
	var t_peak = 0.7

	var y_at_t = func(t):
		# parabola passing through (0, 0) and point (t_peak, peak_y).
		return -(peak_y / t_peak / t_peak) * (t - t_peak) ** 2 + peak_y

	var y_tween = self.create_tween()
	# Abuse of tween easing. You can make something move in a parabola using quadratic easing.
	# Recall that the peak and any other point defines a parabola (among other definitions).
	y_tween.tween_property(self, "offset:y", self.offset.y + peak_y, t_peak).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
	y_tween.tween_property(self, "offset:y", self.offset.y + y_at_t.call(t_end), t_end - t_peak).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_IN)
	# Honestly this might be replaced by rising linearly and then hanging for a little.

	const t_fade = 0.9
	var fade_tween = self.create_tween()
	fade_tween.tween_interval(t_fade)

	# # avoid flashing, even if its very small flashing.
	#fade_tween.tween_property(self, "modulate:a", 0, t_end - t_fade)
	#fade_tween.parallel().tween_property(self, "outline_modulate:a", 0, t_end - t_fade)

	var i = t_fade
	while i < t_end:
		fade_tween.tween_callback(self.hide)
		fade_tween.tween_interval(1.0 / 20.0)
		fade_tween.tween_callback(self.show)
		fade_tween.tween_interval(1.0 / 20.0)
		
		i += 2.0/20.0
	
	fade_tween.tween_callback(self.queue_free)
