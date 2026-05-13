// SPDX-License-Identifier: GPL-2.0-only

#include <kunit/test.h>

#include "shuttlepro-report.h"

static void shuttlepro_kunit_short_report(struct kunit *test)
{
	unsigned char data[SHUTTLEPRO_REPORT_LEN] = { 0 };
	struct shuttlepro_report_state state = { 0 };
	struct shuttlepro_report report = { 0 };

	KUNIT_EXPECT_FALSE(test, shuttlepro_decode_report(data,
							  sizeof(data) - 1,
							  &state, &report));
}

static void shuttlepro_kunit_shuttle_clamp(struct kunit *test)
{
	unsigned char data[SHUTTLEPRO_REPORT_LEN] = { 0xf8, 0, 0, 0, 0 };
	struct shuttlepro_report_state state = { 0 };
	struct shuttlepro_report report = { 0 };

	KUNIT_ASSERT_TRUE(test, shuttlepro_decode_report(data, sizeof(data),
							 &state, &report));
	KUNIT_EXPECT_EQ(test, report.shuttle, SHUTTLEPRO_SHUTTLE_MIN);

	data[0] = 0x08;
	KUNIT_ASSERT_TRUE(test, shuttlepro_decode_report(data, sizeof(data),
							 &state, &report));
	KUNIT_EXPECT_EQ(test, report.shuttle, SHUTTLEPRO_SHUTTLE_MAX);
}

static void shuttlepro_kunit_jog_baseline(struct kunit *test)
{
	unsigned char first[SHUTTLEPRO_REPORT_LEN] = { 0, 10, 0, 0, 0 };
	unsigned char second[SHUTTLEPRO_REPORT_LEN] = { 0, 11, 0, 0, 0 };
	struct shuttlepro_report_state state = { 0 };
	struct shuttlepro_report report = { 0 };

	KUNIT_ASSERT_TRUE(test, shuttlepro_decode_report(first, sizeof(first),
							 &state, &report));
	KUNIT_EXPECT_FALSE(test, report.has_jog_delta);
	KUNIT_EXPECT_EQ(test, report.jog_delta, 0);

	KUNIT_ASSERT_TRUE(test, shuttlepro_decode_report(second, sizeof(second),
							 &state, &report));
	KUNIT_EXPECT_TRUE(test, report.has_jog_delta);
	KUNIT_EXPECT_EQ(test, report.jog_delta, 1);
}

static void shuttlepro_kunit_jog_wraparound(struct kunit *test)
{
	unsigned char first[SHUTTLEPRO_REPORT_LEN] = { 0, 255, 0, 0, 0 };
	unsigned char second[SHUTTLEPRO_REPORT_LEN] = { 0, 0, 0, 0, 0 };
	unsigned char third[SHUTTLEPRO_REPORT_LEN] = { 0, 255, 0, 0, 0 };
	struct shuttlepro_report_state state = { 0 };
	struct shuttlepro_report report = { 0 };

	KUNIT_ASSERT_TRUE(test, shuttlepro_decode_report(first, sizeof(first),
							 &state, &report));
	KUNIT_ASSERT_TRUE(test, shuttlepro_decode_report(second, sizeof(second),
							 &state, &report));
	KUNIT_EXPECT_EQ(test, report.jog_delta, 1);

	KUNIT_ASSERT_TRUE(test, shuttlepro_decode_report(third, sizeof(third),
							 &state, &report));
	KUNIT_EXPECT_EQ(test, report.jog_delta, -1);
}

static void shuttlepro_kunit_button_mask(struct kunit *test)
{
	unsigned char data[SHUTTLEPRO_REPORT_LEN] = { 0, 0, 0, 0xff, 0xff };
	struct shuttlepro_report_state state = { 0 };
	struct shuttlepro_report report = { 0 };

	KUNIT_ASSERT_TRUE(test, shuttlepro_decode_report(data, sizeof(data),
							 &state, &report));
	KUNIT_EXPECT_EQ(test, report.buttons, SHUTTLEPRO_BUTTON_MASK);
}

static struct kunit_case shuttlepro_kunit_cases[] = {
	KUNIT_CASE(shuttlepro_kunit_short_report),
	KUNIT_CASE(shuttlepro_kunit_shuttle_clamp),
	KUNIT_CASE(shuttlepro_kunit_jog_baseline),
	KUNIT_CASE(shuttlepro_kunit_jog_wraparound),
	KUNIT_CASE(shuttlepro_kunit_button_mask),
	{ }
};

static struct kunit_suite shuttlepro_kunit_suite = {
	.name = "hid_shuttlepro",
	.test_cases = shuttlepro_kunit_cases,
};

kunit_test_suite(shuttlepro_kunit_suite);

MODULE_AUTHOR("Robert Balm");
MODULE_DESCRIPTION("Contour ShuttlePro v2 parser KUnit tests");
MODULE_LICENSE("GPL");
