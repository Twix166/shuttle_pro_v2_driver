// SPDX-License-Identifier: GPL-2.0
/*
 * Contour ShuttlePro v2 HID driver.
 *
 * The device sends one 5-byte input report:
 *   byte 0: spring-loaded shuttle wheel, signed -7..7
 *   byte 1: jog wheel position, unsigned 8-bit counter
 *   byte 2: constant/padding according to the HID descriptor
 *   byte 3: buttons 0..7
 *   byte 4: buttons 8..12 in bits 0..4
 */

#include <linux/hid.h>
#include <linux/input.h>
#include <linux/kernel.h>
#include <linux/module.h>
#include <linux/slab.h>

#define USB_VENDOR_ID_CONTOUR		0x0b33
#define USB_DEVICE_ID_SHUTTLEPRO_V2	0x0030

#define SHUTTLEPRO_REPORT_LEN		5
#define SHUTTLEPRO_BUTTONS		13
#define SHUTTLEPRO_BUTTON_BASE		BTN_TRIGGER_HAPPY1
#define SHUTTLEPRO_SHUTTLE_MIN		(-7)
#define SHUTTLEPRO_SHUTTLE_MAX		7

struct shuttlepro {
	struct input_dev *input;
	u16 buttons;
	u8 jog;
	bool have_jog;
};

static int shuttlepro_input_open(struct input_dev *input)
{
	struct hid_device *hdev = input_get_drvdata(input);

	return hid_hw_open(hdev);
}

static void shuttlepro_input_close(struct input_dev *input)
{
	struct hid_device *hdev = input_get_drvdata(input);

	hid_hw_close(hdev);
}

static void shuttlepro_report_buttons(struct shuttlepro *shuttle, u16 buttons)
{
	struct input_dev *input = shuttle->input;
	unsigned int i;
	u16 changed = shuttle->buttons ^ buttons;

	for (i = 0; i < SHUTTLEPRO_BUTTONS; i++) {
		if (changed & BIT(i))
			input_report_key(input, SHUTTLEPRO_BUTTON_BASE + i,
					 buttons & BIT(i));
	}

	shuttle->buttons = buttons;
}

static int shuttlepro_raw_event(struct hid_device *hdev,
				struct hid_report *report, u8 *data, int size)
{
	struct shuttlepro *shuttle = hid_get_drvdata(hdev);
	struct input_dev *input;
	s8 spring;
	u16 buttons;
	int delta;

	if (!shuttle || size < SHUTTLEPRO_REPORT_LEN)
		return 0;

	input = shuttle->input;
	spring = clamp_t(s8, (s8)data[0],
			 SHUTTLEPRO_SHUTTLE_MIN, SHUTTLEPRO_SHUTTLE_MAX);
	buttons = data[3] | ((data[4] & 0x1f) << 8);

	input_report_abs(input, ABS_MISC, spring);

	if (!shuttle->have_jog) {
		shuttle->jog = data[1];
		shuttle->have_jog = true;
	} else {
		delta = (s8)(data[1] - shuttle->jog);
		if (delta)
			input_report_rel(input, REL_DIAL, delta);
		shuttle->jog = data[1];
	}

	shuttlepro_report_buttons(shuttle, buttons);
	input_sync(input);

	return 1;
}

static int shuttlepro_probe(struct hid_device *hdev,
			    const struct hid_device_id *id)
{
	struct shuttlepro *shuttle;
	struct input_dev *input;
	int error;
	int i;

	shuttle = devm_kzalloc(&hdev->dev, sizeof(*shuttle), GFP_KERNEL);
	if (!shuttle)
		return -ENOMEM;

	input = devm_input_allocate_device(&hdev->dev);
	if (!input)
		return -ENOMEM;

	hid_set_drvdata(hdev, shuttle);
	shuttle->input = input;

	error = hid_parse(hdev);
	if (error) {
		hid_err(hdev, "parse failed: %d\n", error);
		return error;
	}

	error = hid_hw_start(hdev, HID_CONNECT_HIDRAW);
	if (error) {
		hid_err(hdev, "hw start failed: %d\n", error);
		return error;
	}

	input->name = "Contour ShuttlePro v2";
	input->phys = hdev->phys;
	input->uniq = hdev->uniq;
	input->id.bustype = hdev->bus;
	input->id.vendor = hdev->vendor;
	input->id.product = hdev->product;
	input->id.version = hdev->version;
	input->dev.parent = &hdev->dev;
	input->open = shuttlepro_input_open;
	input->close = shuttlepro_input_close;

	input_set_drvdata(input, hdev);
	input_set_capability(input, EV_REL, REL_DIAL);
	input_set_abs_params(input, ABS_MISC, SHUTTLEPRO_SHUTTLE_MIN,
			     SHUTTLEPRO_SHUTTLE_MAX, 0, 0);

	for (i = 0; i < SHUTTLEPRO_BUTTONS; i++)
		input_set_capability(input, EV_KEY,
				     SHUTTLEPRO_BUTTON_BASE + i);

	error = input_register_device(input);
	if (error) {
		hid_err(hdev, "input register failed: %d\n", error);
		hid_hw_stop(hdev);
		return error;
	}

	hid_info(hdev, "Contour ShuttlePro v2 driver loaded\n");

	return 0;
}

static void shuttlepro_remove(struct hid_device *hdev)
{
	hid_hw_stop(hdev);
}

static const struct hid_device_id shuttlepro_devices[] = {
	{ HID_USB_DEVICE(USB_VENDOR_ID_CONTOUR, USB_DEVICE_ID_SHUTTLEPRO_V2) },
	{ }
};
MODULE_DEVICE_TABLE(hid, shuttlepro_devices);

static struct hid_driver shuttlepro_driver = {
	.name = "hid-shuttlepro",
	.id_table = shuttlepro_devices,
	.probe = shuttlepro_probe,
	.remove = shuttlepro_remove,
	.raw_event = shuttlepro_raw_event,
};
module_hid_driver(shuttlepro_driver);

MODULE_AUTHOR("Codex");
MODULE_DESCRIPTION("Contour ShuttlePro v2 HID driver");
MODULE_LICENSE("GPL");
