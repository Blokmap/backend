SELECT
	l.*,
	description.*,
	excerpt.*,
	approver.*,
	rejecter.*,
	creator.*,
	updater.*,
	op_time.*,
	tag.*,
	img.*
FROM
	location l
	INNER JOIN translation description
		ON l.description_id = description.id
	INNER JOIN translation excerpt
		ON l.excerpt_id = excerpt.id
	LEFT OUTER JOIN simple_profile approver
		ON {include_approved_by} = true AND l.approved_by = approver.id
	LEFT OUTER JOIN simple_profile rejecter
		ON {include_rejected_by} = true AND l.rejected_by = rejecter.id
	LEFT OUTER JOIN simple_profile creator
		ON {include_created_by} = true AND l.created_by = creator.id
	LEFT OUTER JOIN simple_profile updater
		ON {include_updated_by} = true AND l.updated_by = updater.id
	LEFT OUTER JOIN opening_time op_time
		ON op_time.location_id = l.id
	LEFT OUTER JOIN location_tag l_tag
		ON {include_tags} AND l_tag.location_id = l.id
	LEFT OUTER JOIN tag
		ON {include_tags} AND l_tags.tag_id = tag.id
	LEFT OUTER JOIN location_image l_img
		ON {include_images} AND l_img.location_id = l.id
	LEFT OUTER JOIN image img
		ON {include_images} AND l_img.image_id = img.id
WHERE
	(
		l.name % {query}
		OR description.{lang} % {query}
		OR excerpt.{lang} % {query}
	)
	AND ({distance} <= fancy_cirkel_berekening({center_lat}, {center_lng}))
	AND l.is_reservable = {is_reservable}
	AND (
		op_time.day = {open_on_day}
		AND (
			{open_on_time} BETWEEN op_time.start_time AND op_time.end_time
		)
		AND op_time.id IS NOT NULL
	)
	AND (
		(
			l.latitude BETWEEN {south_lat} AND {north_lat}
		)
		AND (
			l.longitude BETWEEN {south_lng} AND {north_lng}
		)
	)
ORDER BY l.id
LIMIT {limit}
OFFSET {offset}
