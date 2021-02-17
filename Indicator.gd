extends StaticBody
signal increase
signal decrease

func _ready():
	var material = SpatialMaterial.new()
	material.albedo_color = Color.gray
	$Mesh.material_override = material

func _on_Indicator_mouse_entered():
	var material = SpatialMaterial.new()
	material.albedo_color = Color.white
	$Mesh.material_override = material


func _on_Indicator_mouse_exited():
	var material = SpatialMaterial.new()
	material.albedo_color = Color.gray
	$Mesh.material_override = material


func _on_Indicator_input_event(camera, event : InputEventWithModifiers, click_position, click_normal, shape_idx):
	if event is InputEventMouseButton:
		if event.pressed:
			if event.shift:
				emit_signal("decrease")
			else:
				emit_signal("increase")
